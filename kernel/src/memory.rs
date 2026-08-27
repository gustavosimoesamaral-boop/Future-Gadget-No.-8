use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use bootloader_api::BootInfo;
use linked_list_allocator::LockedHeap;
use x86_64::{
    PhysAddr,
    VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        Mapper,
        Page,
        PageTableFlags,
        FrameAllocator,
        PageTable,
        PageSize,
        PhysFrame,
        Size4KiB,
        mapper::{
            MapToError,
            OffsetPageTable,
        },
    },
};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
pub const HEAP_START: u64 = 0x_4444_4444_0000;
pub const HEAP_SIZE: u64 = 100 * 1024; // 100 KiB

pub fn usable_memory_bytes(memory_regions: &MemoryRegions) -> u64 {
    let mut total = 0;

    for region in memory_regions.iter() {
        if matches!(region.kind, MemoryRegionKind::Usable) {
            total += region.end - region.start;
        }
    }

    total
}

pub unsafe fn init_mapper(
    physical_memory_offset: u64,
) -> OffsetPageTable<'static> {
    let (level_4_table_frame, _) = Cr3::read();

    let physical_address = level_4_table_frame.start_address();

    let virtual_address =
        VirtAddr::new(physical_memory_offset + physical_address.as_u64());

    let page_table =
        &mut *virtual_address.as_mut_ptr::<PageTable>();

    OffsetPageTable::new(
        page_table,
        VirtAddr::new(physical_memory_offset),
    )
}

pub fn init_heap(
    mapper: &mut OffsetPageTable<'static>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let heap_start = VirtAddr::new(HEAP_START);
    let heap_end = heap_start + HEAP_SIZE - 1u64;

    let start_page = Page::containing_address(heap_start);
    let end_page = Page::containing_address(heap_end);

    let page_range = Page::range_inclusive(start_page, end_page);

    let flags =
        PageTableFlags::PRESENT |
        PageTableFlags::WRITABLE;

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        unsafe {
            mapper.map_to(
                page,
                frame,
                flags,
                frame_allocator,
            )?
            .flush();
        }
    }

    unsafe {
        ALLOCATOR
            .lock()
            .init(
                HEAP_START as *mut u8,
                HEAP_SIZE as usize,
            );
    }

    Ok(())
}

pub struct BootInfoFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub fn init(memory_regions: &'static MemoryRegions) -> Self {
        Self {
            memory_regions,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        self.memory_regions
            .iter()
            .filter(|region| matches!(region.kind, MemoryRegionKind::Usable))
            .flat_map(|region| {
                let start = core::cmp::max(
                    0x1000,
                    (region.start + Size4KiB::SIZE - 1)
                        & !(Size4KiB::SIZE - 1),
                );

                let end = region.end & !(Size4KiB::SIZE - 1);

                (start..end)
                    .step_by(Size4KiB::SIZE as usize)
                    .filter_map(|address| {
                        PhysFrame::from_start_address(
                            PhysAddr::new(address),
                        )
                        .ok()
                    })
            })
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);

        if frame.is_some() {
            self.next += 1;
        }

        frame
    }
}

pub struct MemoryManager {
    mapper: OffsetPageTable<'static>,
    frame_allocator: BootInfoFrameAllocator,
}

impl MemoryManager {
    pub unsafe fn init(
        memory_regions: &'static bootloader_api::info::MemoryRegions,
        physical_memory_offset: u64,
    ) -> Self {

        let frame_allocator =
            BootInfoFrameAllocator::init(memory_regions);

        let mapper =
            init_mapper(physical_memory_offset);

        Self {
            mapper,
            frame_allocator,
        }
    }

    pub fn allocate_frame(
        &mut self,
    ) -> Option<PhysFrame<Size4KiB>> {
        self.frame_allocator.allocate_frame()
    }

    pub fn init_heap(
        &mut self,
    ) -> Result<(), MapToError<Size4KiB>> {
        init_heap(
            &mut self.mapper,
            &mut self.frame_allocator,
        )
    }
}