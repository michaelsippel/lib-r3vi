use {
    crate::{
        view::{
            InnerViewPort, OuterViewPort, View, Observer, ViewPort,
            list::*
        }
    },
    std::sync::RwLock,
    std::{
        ops::{Deref, DerefMut},
        sync::Arc,
    },
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum VecDiff<T> {
    Clear,
    Push(T),
    Remove(usize),
    Insert { idx: usize, val: T },
    Update { idx: usize, val: T },
}

impl<T> View for Vec<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Msg = VecDiff<T>;
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct VecBufferTarget<T>
where T: Clone + Send + Sync + 'static
{
    buffer: VecBuffer<T>,
    src_view: Option<Arc<dyn ListView<T>>>
}

impl<T> Observer< dyn ListView<T> > for VecBufferTarget<T>
where T: Clone + Send + Sync + 'static
{
    fn notify(&mut self, msg: &ListDiff<T>) {
        self.buffer.apply_diff(
            match msg.clone() {
                ListDiff::Clear => VecDiff::Clear,
                ListDiff::Remove(idx) => VecDiff::Remove(idx),
                ListDiff::Insert{ idx, val } => VecDiff::Insert{ idx, val },
                ListDiff::Update{ idx, val } => VecDiff::Update{ idx, val }
            }
        );
    }

    fn reset(&mut self, src_view: Option<Arc<dyn ListView<T>>>) {
        self.src_view = src_view;

        // copy all elements of new view
        if let Some(v) = self.src_view.as_ref() {
            self.buffer.clear();
            for x in v.iter() {
                self.buffer.push(x.clone());
            }
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub struct VecBuffer<T>
where
    T: Clone + Send + Sync + 'static,
{
    data: Arc<RwLock<Vec<T>>>,
    port: InnerViewPort<RwLock<Vec<T>>>
}

impl<T> VecBuffer<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn with_data_port(data: Vec<T>, port: InnerViewPort<RwLock<Vec<T>>>) -> Self {
        let data = Arc::new(RwLock::new(data));
        port.set_view(Some(data.clone()));

        for x in data.read().unwrap().iter().cloned() {
            port.notify(&VecDiff::Push(x));
        }
            
        VecBuffer {
            data,
            port
        }
    }

    pub fn attach_to(&self, port: OuterViewPort< dyn ListView<T> >) -> Arc<RwLock<VecBufferTarget<T>>> {
        self.port.0.add_update_hook(Arc::new(port.0.clone()));

        let target = Arc::new(RwLock::new(
            VecBufferTarget {
                buffer: self.clone(),
                src_view: None
            }
        ));
        port.add_observer(target.clone());

        target
    }

    pub fn with_data(data: Vec<T>) -> Self {
        VecBuffer::with_data_port(data, ViewPort::new().into_inner())
    }
    
    pub fn with_port(port: InnerViewPort<RwLock<Vec<T>>>) -> Self {
        VecBuffer::with_data_port(vec![], port)
    }

    pub fn new() -> Self {
        VecBuffer::with_port(ViewPort::new().into_inner())
    }

    pub fn get_port(&self) -> OuterViewPort<RwLock<Vec<T>>> {
        self.port.0.outer()
    }

    pub fn apply_diff(&mut self, diff: VecDiff<T>) {
        let mut data = self.data.write().unwrap();
        match &diff {
            VecDiff::Clear => {
                data.clear();
            }
            VecDiff::Push(val) => {
                data.push(val.clone());
            }
            VecDiff::Remove(idx) => {
                data.remove(*idx);
            }
            VecDiff::Insert { idx, val } => {
                data.insert(*idx, val.clone());
            }
            VecDiff::Update { idx, val } => {
                data[*idx] = val.clone();
            }
        }
        drop(data);

        self.port.notify(&diff);
    }

    pub fn len(&self) -> usize {
        self.data.read().unwrap().len()
    }

    pub fn get(&self, idx: usize) -> T {
        self.data.read().unwrap()[idx].clone()
    }

    pub fn clear(&mut self) {
        self.apply_diff(VecDiff::Clear);
    }

    pub fn push(&mut self, val: T) {
        self.apply_diff(VecDiff::Push(val));
    }

    pub fn remove(&mut self, idx: usize) {
        self.apply_diff(VecDiff::Remove(idx));
    }

    pub fn insert(&mut self, idx: usize, val: T) {
        self.apply_diff(VecDiff::Insert { idx, val });
    }

    pub fn update(&mut self, idx: usize, val: T) {
        self.apply_diff(VecDiff::Update { idx, val });
    }

    pub fn get_mut(&mut self, idx: usize) -> MutableVecAccess<T> {
        MutableVecAccess {
            buf: self.clone(),
            idx,
            val: self.get(idx),
        }
    }

    pub fn into_inner(self) -> Arc<RwLock<Vec<T>>> {
        self.data
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct MutableVecAccess<T>
where
    T: Clone + Send + Sync + 'static,
{
    buf: VecBuffer<T>,
    idx: usize,
    val: T,
}

impl<T> Deref for MutableVecAccess<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.val
    }
}

impl<T> DerefMut for MutableVecAccess<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl<T> Drop for MutableVecAccess<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        self.buf.update(self.idx, self.val.clone());
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[cfg(test)]
mod tests {
    use crate::{
        buffer::vec::*,
        view::{
            port::UpdateTask,
            list::ListView,
        }
    };

    #[test]
    fn vec_buffer1() {
        let mut buffer = VecBuffer::new();
        
        buffer.push('a');
        buffer.push('b');
        buffer.push('c');
    }

    #[test]
    fn list_to_vec() {
        let mut buf = VecBuffer::<char>::new();
        let list_view = buf.get_port().to_list();

        let mut buf2 = VecBuffer::new();
        let keepalive = buf2.attach_to( list_view.clone() );

        assert_eq!(buf2.len(), 0);

        buf.push('a');

        buf2.get_port().0.update();
        assert_eq!(buf2.len(), 1);
        assert_eq!(buf2.get(0), 'a');

        buf.push('b');

        list_view.0.update();
        assert_eq!(buf2.len(), 2);
        assert_eq!(buf2.get(0), 'a');
        assert_eq!(buf2.get(1), 'b');

        buf.push('c');
        buf.remove(0);

        list_view.0.update();
        assert_eq!(buf2.len(), 2);
        assert_eq!(buf2.get(0), 'b');
        assert_eq!(buf2.get(1), 'c');
    }
}

