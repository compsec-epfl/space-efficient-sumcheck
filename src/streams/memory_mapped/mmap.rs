use ark_ff::Field;
use ark_std::{fs::File, io::Cursor, marker::PhantomData};
use memmap2::Mmap;

use crate::streams::EvaluationStream;

/*
 * We want to run sumcheck over some known polynomials, so we can use this
 * stream to pass in a vector containing evaluations of the polynomial
 * from 0..n^2
 */

#[derive(Debug)]
pub struct MmapEvaluationStream<F: Field> {
    mmap: Mmap,
    value_size: usize,
    _field: PhantomData<F>,
}

impl<F: Field> MmapEvaluationStream<F> {
    pub fn new(path: String) -> Self {
        let file = File::open(path).expect("Failed to open the file for stream");
        let mmap = unsafe { Mmap::map(&file).expect("failed to map the file") };    
        Self {
            mmap,
            value_size: 256, // TODO: an optimization is to make this smaller
            _field: PhantomData,
        }
    }
    fn bytes_to_field(bytes: &[u8]) -> Option<F> {
        F::deserialize_compressed(&mut Cursor::new(bytes)).ok()
    }
    fn read_index(&self, index: usize) -> Option<F> {
        let start = index * self.value_size;
        let end = start + self.value_size;
        if end > self.mmap.len() {
            return None;
        }
        let bytes: &[u8] = &self.mmap[start..end];
        Self::bytes_to_field(bytes)
    }
}

impl<F: Field> EvaluationStream<F> for MmapEvaluationStream<F> {
    fn claim(&self) -> F {
        let mut sum = F::ZERO;
        for i in 0..self.mmap.len() {
            sum += self.read_index(i).unwrap();
        }
        sum
    }
    fn evaluation(&self, point: usize) -> F {
        self.read_index(point).unwrap()
    }
    fn num_variables(&self) -> usize {
        self.mmap.len().ilog2() as usize
    }
}
