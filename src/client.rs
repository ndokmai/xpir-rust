use libc;
use std::slice;
use std::mem;

use super::{PirQuery, PirReply};

extern "C" {
    fn cpp_client_setup(
        len: u64, 
        num: u64, 
        alpha: u64, 
        depth: u64, 
    ) -> *mut libc::c_void;

    fn cpp_client_generate_query(
        client: *const libc::c_void,
        index: u64, 
        q_len: *mut u64, 
        q_num: *mut u64, 
    ) -> *mut u8; 

    fn cpp_client_process_reply(
        client: *const libc::c_void,
        reply: *const u8, 
        r_len: u64, 
        r_num: u64, 
        e_len: *mut u64, 
    ) -> *mut u8;

    fn cpp_client_free(client: *mut libc::c_void);

    fn cpp_client_update_db_params(
        client: *const libc::c_void,
        len: u64, 
        num: u64, 
        alpha: u64, 
        depth: u64, 
    );
    fn cpp_buffer_free(buffer: *mut u8);
}

pub struct PirClient<'a> {
    client: &'a mut libc::c_void,
    num: u64,
}

impl<'a> Drop for PirClient<'a> {
    fn drop(&mut self) {
        unsafe {
            cpp_client_free(self.client);
        }
    }
}

impl<'a> PirClient<'a> {
    pub fn new(size: u64, num: u64) -> PirClient<'a> {
        //Default: alpha = 8, depth = 2.
        let client_ptr: &'a mut libc::c_void =
            unsafe { &mut *(cpp_client_setup(size * num, num, 8, 2)) };

        PirClient {
            client: client_ptr,
            num: num,
        }
    }

    pub fn with_params(size: u64, num: u64, alpha: u64, depth: u64) -> PirClient<'a> {
        let client_ptr: &'a mut libc::c_void =
            unsafe { &mut *(cpp_client_setup(size * num, num, alpha, depth)) };

        PirClient {
            client: client_ptr,
            num: num,
        }
    }

    pub fn update_params(&mut self, size: u64, num: u64, alpha: u64, depth: u64) {
        unsafe {
            cpp_client_update_db_params(self.client, size * num, num, alpha, depth);
        }

        self.num = num;
    }

    pub fn gen_query(&self, index: u64) -> PirQuery {
        assert!(index <= self.num);

        let mut q_len: u64 = 0;
        let mut q_num: u64 = 0;

        let query: Vec<u8> = unsafe {
            let ptr = cpp_client_generate_query(self.client, index, &mut q_len, &mut q_num);
            let q = slice::from_raw_parts_mut(ptr as *mut u8, q_len as usize).to_vec();
            cpp_buffer_free(ptr);
            q
        };

        PirQuery {
            query: query,
            num: q_num,
        }
    }

    pub fn decode_reply<T>(&self, reply: &PirReply) -> T
    where
        T: Clone,
    {
        let mut e_len: u64 = 0;

        let result: T = unsafe {
            let ptr = cpp_client_process_reply(
                self.client,
                reply.reply.as_ptr(),
                reply.reply.len() as u64,
                reply.num,
                &mut e_len,
            );
            assert_eq!(e_len as usize, mem::size_of::<T>());
            let r = slice::from_raw_parts_mut(ptr as *mut T, 1).to_vec();
            cpp_buffer_free(ptr);
            r[0].clone()
        };

        result
    }

    pub fn decode_reply_to_vec(&self, reply: &PirReply) -> Vec<u8> 
    {
        let mut e_len: u64 = 0;
        let result: Vec<u8> = unsafe {
            let ptr = cpp_client_process_reply(
                self.client,
                reply.reply.as_ptr(),
                reply.reply.len() as u64,
                reply.num,
                &mut e_len,
            );
            let r = slice::from_raw_parts_mut(ptr as *mut u8, e_len as usize).to_vec();
            cpp_buffer_free(ptr);
            r
        };
        result
    }
}
