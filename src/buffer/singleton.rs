use {
    crate::{
        view::{
            Observer,
            InnerViewPort, OuterViewPort, View, ViewPort,
            singleton::SingletonView
        },
    },
    std::sync::RwLock,
    std::{
        ops::{Deref, DerefMut},
        sync::Arc,
    },
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct SingletonBufferView<T: Clone + Send + Sync + 'static>(pub Arc<RwLock<T>>);

impl<T> View for SingletonBufferView<T>
where
    T: Clone + Send + Sync + 'static
{
    type Msg = ();
}

impl<T> SingletonView for SingletonBufferView<T>
where
    T: Clone + Send + Sync + 'static
{
    type Item = T;

    fn get(&self) -> Self::Item {
        self.0.read().unwrap().clone()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub struct SingletonBuffer<T>
where
    T: Clone + Send + Sync + 'static
{
    pub value: Arc<RwLock<T>>,
    pub port: InnerViewPort<dyn SingletonView<Item = T>>
}

pub struct SingletonBufferTarget<T>
where
    T: Clone + Send + Sync + 'static
{
    buffer: SingletonBuffer<T>,
    src_view: Option<Arc<dyn SingletonView<Item = T>>>
}

impl<T> Observer< dyn SingletonView<Item = T> > for SingletonBufferTarget<T>
where
    T: Clone + Send + Sync + 'static
{
    fn notify(&mut self, _msg: &()) {
        if let Some(v) = self.src_view.clone() {
            self.buffer.set( v.get() );
        }
    }

    fn reset(&mut self, view: Option<Arc<dyn SingletonView<Item = T>>>) {
        self.src_view = view;
        self.notify(&());
    }
}

impl<T> SingletonBuffer<T>
where
    T: Clone + Send + Sync + 'static
{
    pub fn with_port(value: T, port: InnerViewPort<dyn SingletonView<Item = T>>) -> Self {
        let value = Arc::new(RwLock::new(value));
        port.set_view(Some(Arc::new(SingletonBufferView(value.clone()))));

        SingletonBuffer {
            value,
            port
        }
    }

    pub fn attach_to(&self, port: OuterViewPort<dyn SingletonView<Item = T>>) -> Arc<RwLock<SingletonBufferTarget<T>>> {
        self.port.0.add_update_hook(Arc::new(port.0.clone()));

        let target = Arc::new(RwLock::new(
            SingletonBufferTarget {
                buffer: self.clone(),
                src_view: None
            }
        ));

        port.add_observer(target.clone());
        target
    }

    pub fn make_in_port(&self) -> InnerViewPort<dyn SingletonView<Item = T>> {
        let port = ViewPort::new();
        self.attach_to(port.outer());
        port.into_inner()
    }

    pub fn new(value: T) -> Self {
        SingletonBuffer::with_port(value, ViewPort::new().into_inner())
    }

    pub fn get_port(&self) -> OuterViewPort<dyn SingletonView<Item = T>> {
        self.port.0.outer()
    }

    pub fn get(&self) -> T {
        self.value.read().unwrap().clone()
    }

    pub fn get_mut(&self) -> MutableSingletonAccess<T> {
        MutableSingletonAccess {
            buf: self.clone(),
            val: self.get(),
        }
    }

    pub fn set(&mut self, new_value: T) {
        let mut v = self.value.write().unwrap();
        *v = new_value;
        drop(v);
        self.port.notify(&());
    }

    pub fn into_inner(self) -> Arc<RwLock<T>> {
        self.value
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct MutableSingletonAccess<T>
where
    T: Clone + Send + Sync + 'static,
{
    buf: SingletonBuffer<T>,
    val: T,
}

impl<T> Deref for MutableSingletonAccess<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.val
    }
}

impl<T> DerefMut for MutableSingletonAccess<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl<T> Drop for MutableSingletonAccess<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        self.buf.set(self.val.clone());
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[cfg(test)]
mod tests {
    use crate::buffer::singleton::*;
    
    #[test]
    fn singleton_buffer1() {
        let buffer = SingletonBuffer::<char>::new('a');
        let port = buffer.get_port();

        assert_eq!(buffer.get(), 'a');
        assert_eq!(port.get_view().get(), 'a');

        *buffer.get_mut() = 'b';
        assert_eq!(buffer.get(), 'b');
        assert_eq!(port.get_view().get(), 'b');
    }
}

