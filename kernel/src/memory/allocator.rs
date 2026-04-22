use core::ptr::null_mut;
use alloc::alloc::{GlobalAlloc, Layout};
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB
    },
    VirtAddr,
};
use crate::{Locked, println};

pub const HEAP_START: usize = 0x4242_0000_0000;
pub const HEAP_SIZE: usize = 128 * 1024; // 128 KiB
pub const PAGE_SIZE: usize = 4096;
const BLOCK_SIZES: [usize; 9] = [8, 16, 32, 64, 128, 256, 512, 1024, 2048];

#[global_allocator]
static ALLOCATOR: Locked<BlockAllocator> = Locked::new(BlockAllocator::new());

struct Node {
    next: Option<&'static mut Node>,
}

struct PageAllocator {
    pages: Option<&'static mut Node>,
    watermark: usize,
}

impl PageAllocator {
    pub const fn new() -> Self {
        Self {
            pages: None,
            watermark: 0,
        }
    }

    unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.watermark = heap_start;
        // TODO: end of heap don't go past that....
    }

    unsafe fn allocate(&mut self, layout: Layout) -> *mut u8 {
        let size = PAGE_SIZE * (layout.size().div_ceil(PAGE_SIZE));
        
        let ptr = if size == PAGE_SIZE && let Some(page) = self.pages.take() {
            page as *mut Node as *mut u8
        }
        else {
            let old = self.watermark;
            self.watermark += size;
            println!("old watermark: {:0X}, new watermark: {:0X}",old, self.watermark);
            old as *mut u8
        };
        
        
        println!("allocating page {layout:#?} calc size: {size}, returning: 0x{:0X}", ptr.addr());
        ptr
    }

    unsafe fn deallocate(&mut self, ptr: *mut u8, layout: Layout) {
        println!("deallocating page: {:0X}, layout: {layout:#?}", ptr.addr());
        let size = layout.size().div_ceil(PAGE_SIZE);
        for i in 0..size {
            let new_node = Node {
                next: self.pages.take()
            };

            let new_node_ptr = unsafe { ptr.byte_offset((i * PAGE_SIZE) as isize) } as *mut Node;
            unsafe {
                new_node_ptr.write(new_node);
                self.pages = Some(&mut *new_node_ptr);
            }
        }
    }
}

struct BlockAllocator {
    blocks: [Option<&'static mut Node>; BLOCK_SIZES.len()],
    page_allocator: PageAllocator,
}

impl BlockAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut Node> = None;
        Self {
            blocks: [EMPTY; BLOCK_SIZES.len()],
            page_allocator: PageAllocator::new()
        }
    }

    unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        unsafe { self.page_allocator.init(heap_start, heap_size); }
    }

    fn list_index(layout: &Layout) -> Option<usize> {
        let block_size = layout.size().max(layout.align());
        BLOCK_SIZES.iter().position(|&s| s >= block_size)
    }
}

unsafe impl GlobalAlloc for Locked<BlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        let ptr = match BlockAllocator::list_index(&layout) {
            Some(index) => {
                match allocator.blocks[index].take() {
                    Some(node) => {
                        allocator.blocks[index] = node.next.take();
                        node as *mut Node as *mut u8
                    },
                    None => {
                        // FIXME: won't work if block size isn't power of two
                        let size = BLOCK_SIZES[index];
                        let align = BLOCK_SIZES[index];
                        let layout = Layout::from_size_align(size, align).expect("constructing layout failed"); 
                        unsafe { allocator.page_allocator.allocate(layout) }
                    }
                }
            },
            None => unsafe { allocator.page_allocator.allocate(layout) },
        };

        println!("allocating block {layout:#?}, returning: 0x{:0X}", ptr.addr());

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            println!("deallocating block: {:0X}, layout: {layout:#?}", ptr.addr());
        let mut allocator = self.lock();
        match BlockAllocator::list_index(&layout) {
            Some(index) => {
                let new_node = Node {
                    next: allocator.blocks[index].take()
                };

                let new_node_ptr = ptr as *mut Node;
                unsafe {
                    new_node_ptr.write(new_node);
                    allocator.blocks[index] = Some(&mut *new_node_ptr);
                }
            },
            None => {
                unsafe { allocator.page_allocator.deallocate(ptr, layout); }
            }
        }
    }
}

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        }
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}