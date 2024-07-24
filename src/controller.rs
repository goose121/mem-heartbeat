use std::ptr;

/// A controller which takes care of actually allocating the memory
/// required to make a memory heartbeat, as well as writing to each
/// page to ensure that the memory actually gets allocated.
pub struct Controller {
    page_size: usize,
    buf_len: usize,
    buf: *mut u8
}

impl Controller {
    pub fn new(page_size: usize) -> Controller {
        let buf: *mut u8 = unsafe {
            libc::mmap(ptr::null_mut(), 1, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_PRIVATE | libc::MAP_ANONYMOUS, -1, 0)
        } as *mut u8;

        unsafe {
            *buf = 1;
        }
        Controller {
            page_size,
            buf_len: 1,
            buf
        }
    }

    pub fn adjust(&mut self, mut new_len: usize) {
        if new_len == 0 {
            new_len = 1;
        }
        
        unsafe {
            self.buf = libc::mremap(self.buf as *mut libc::c_void, self.buf_len, new_len, libc::MREMAP_MAYMOVE) as *mut u8;
        }

        let start_page = ((self.buf_len + self.page_size) & !self.page_size) / self.page_size;
        let end_page = (new_len & !self.page_size) / self.page_size;

        for page in start_page..end_page {
            unsafe {
                *self.buf.add(page * self.page_size) = 1;
            }
        }

        self.buf_len = new_len;
    }
}

unsafe impl Send for Controller {}
