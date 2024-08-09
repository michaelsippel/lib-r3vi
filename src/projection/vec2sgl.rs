use {
    crate::{
        view::{
            InnerViewPort, Observer, ObserverBroadcast, ObserverExt, OuterViewPort, View, ViewPort,
            sequence::SequenceView,
            singleton::SingletonView
        },
        buffer::vec::VecDiff,
    },
    std::sync::Arc,
    std::sync::RwLock,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

/// Adapter View implementing `Singleton` for `Vec`
pub struct VecSingleton<T>
where
    T: Clone + Send + Sync + 'static,
{
    data: Option<Arc<RwLock<Vec<T>>>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn SingletonView<Item = Vec<T>>>>>,
}

impl<T> VecSingleton<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(port: InnerViewPort<dyn SingletonView<Item = Vec<T>>>) -> Arc<RwLock<Self>> {
        let sgl = Arc::new(RwLock::new(VecSingleton {
            data: None,
            cast: port.get_broadcast(),
        }));
        port.set_view(Some(sgl.clone()));
        sgl
    }
}

impl<T> Observer<RwLock<Vec<T>>> for VecSingleton<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn reset(&mut self, view: Option<Arc<RwLock<Vec<T>>>>) {
        self.data = view;
        self.cast.notify(&());
    }

    fn notify(&mut self, _diff: &VecDiff<T>) {
        self.cast.notify(&());
    }
}

impl<T> View for VecSingleton<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Msg = ();
}

impl<T> SingletonView for VecSingleton<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Item = Vec<T>;

    fn get(&self) -> Vec<T> {
        self.data.as_ref().unwrap().read().unwrap().clone()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<T> OuterViewPort<RwLock<Vec<T>>>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn to_singleton(&self) -> OuterViewPort<dyn SingletonView<Item = Vec<T>>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));

        let vec_sgl = VecSingleton::new(port.inner());
        self.add_observer(vec_sgl.clone());
        port.into_outer()
    }
}
