use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::{
    PhysAddr,
    VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator,
        PageTable,
        PageSize,
        PhysFrame,
        Size4KiB,
        mapper::OffsetPageTable,
    },
};

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