use {
    crate::{
        view::{
            Observer, ObserverBroadcast, OuterViewPort, View, ViewPort,
            singleton::SingletonView,
        }
    },
    std::sync::Arc,
    std::sync::RwLock,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Item: 'static> OuterViewPort<dyn SingletonView<Item = Item>> {
    pub fn map<DstItem: 'static, F: Fn(Item) -> DstItem + Send + Sync + 'static>(
        &self,
        f: F,
    ) -> OuterViewPort<dyn SingletonView<Item = DstItem>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));

        let map = Arc::new(RwLock::new(MapSingleton {
            src_view: None,
            f,
            cast: port.inner().get_broadcast(),
        }));

        self.add_observer(map.clone());
        port.inner().set_view(Some(map));
        port.into_outer()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct MapSingleton<DstItem, SrcView, F>
where
    SrcView: SingletonView + ?Sized,
    F: Fn(SrcView::Item) -> DstItem + Send + Sync,
{
    src_view: Option<Arc<SrcView>>,
    f: F,
    cast: Arc<RwLock<ObserverBroadcast<dyn SingletonView<Item = DstItem>>>>,
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<DstItem, SrcView, F> View for MapSingleton<DstItem, SrcView, F>
where
    SrcView: SingletonView + ?Sized,
    F: Fn(SrcView::Item) -> DstItem + Send + Sync,
{
    type Msg = ();
}

impl<DstItem, SrcView, F> SingletonView for MapSingleton<DstItem, SrcView, F>
where
    SrcView: SingletonView + ?Sized,
    F: Fn(SrcView::Item) -> DstItem + Send + Sync,
{
    type Item = DstItem;

    fn get(&self) -> DstItem {
        (self.f)(self.src_view.as_ref().unwrap().get())
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<DstItem, SrcView, F> Observer<SrcView> for MapSingleton<DstItem, SrcView, F>
where
    SrcView: SingletonView + ?Sized,
    F: Fn(SrcView::Item) -> DstItem + Send + Sync,
{
    fn reset(&mut self, view: Option<Arc<SrcView>>) {
        self.src_view = view;
        self.cast.notify(&());
    }

    fn notify(&mut self, msg: &()) {
        self.cast.notify(msg);
    }
}


//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[cfg(test)]
mod tests {
    use crate::{
        buffer::singleton::*,
        projection::map_singleton::*
    };

    #[test]
    fn singleton_map1() {
        let mut buffer = SingletonBuffer::new(0);

        let src_port = buffer.get_port();
        let dst_port = src_port.map(|x| x + 10);

        let dst_view = dst_port.get_view();

        assert_eq!(dst_view.get(), 10);
        buffer.set(5);
        assert_eq!(dst_view.get(), 15);
    }
}

