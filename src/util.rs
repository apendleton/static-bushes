#[derive(Debug, PartialEq, Eq)]
pub enum IndexVec {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl IndexVec {
    pub fn get(&self, idx: usize) -> u32 {
        match self {
            IndexVec::U16(v) => v[idx] as u32,
            IndexVec::U32(v) => v[idx],
        }
    }

    pub fn set(&mut self, idx: usize, val: u32) {
        match self {
            IndexVec::U16(v) => {
                v[idx] = val as u16;
            }
            IndexVec::U32(v) => {
                v[idx] = val;
            }
        }
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        match self {
            IndexVec::U16(v) => v.swap(i, j),
            IndexVec::U32(v) => v.swap(i, j),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            IndexVec::U16(v) => v.len(),
            IndexVec::U32(v) => v.len(),
        }
    }

    #[cfg(test)]
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = u32> + 'a {
        let out: Box<dyn Iterator<Item = u32>> = match self {
            IndexVec::U16(v) => Box::new(v.iter().map(|x| *x as u32)),
            IndexVec::U32(v) => Box::new(v.iter().cloned()),
        };
        out
    }
}
