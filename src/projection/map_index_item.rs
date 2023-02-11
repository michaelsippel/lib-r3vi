pub use {
    crate::{
        view::{
            InnerViewPort, Observer, ObserverBroadcast, ObserverExt, OuterViewPort, View, ViewPort,
            index::{IndexArea, IndexView},
        },
    },
    std::sync::RwLock,
    std::{boxed::Box, sync::Arc},
};

impl<Key, Item> OuterViewPort<dyn IndexView<Key, Item = Item>>
where
    Key: Clone + Send + Sync + 'static,
    Item: Send + Sync + 'static,
{
    pub fn map_item<DstItem: 'static, F: Fn(&Key, &Item) -> DstItem + Send + Sync + 'static>(
        &self,
        f: F,
    ) -> OuterViewPort<dyn IndexView<Key, Item = DstItem>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));

        let map = MapIndexItem::new(port.inner(), f);
        self.add_observer(map.clone());
        port.into_outer()
    }
}

pub struct MapIndexItem<Key, DstItem, SrcView, F>
where
    Key: Clone + Send + Sync,
    SrcView: IndexView<Key> + ?Sized,
    F: Fn(&Key, &SrcView::Item) -> DstItem + Send + Sync,
{
    src_view: Option<Arc<SrcView>>,
    f: F,
    cast: Arc<RwLock<ObserverBroadcast<dyn IndexView<Key, Item = DstItem>>>>,
}

impl<Key, DstItem, SrcView, F> MapIndexItem<Key, DstItem, SrcView, F>
where
    Key: Clone + Send + Sync + 'static,
    DstItem: 'static,
    SrcView: IndexView<Key> + ?Sized + 'static,
    F: Fn(&Key, &SrcView::Item) -> DstItem + Send + Sync + 'static,
{
    fn new(port: InnerViewPort<dyn IndexView<Key, Item = DstItem>>, f: F) -> Arc<RwLock<Self>> {
        let map = Arc::new(RwLock::new(MapIndexItem {
            src_view: None,
            f,
            cast: port.get_broadcast(),
        }));

        port.set_view(Some(map.clone()));
        map
    }
}

impl<Key, DstItem, SrcView, F> View for MapIndexItem<Key, DstItem, SrcView, F>
where
    Key: Clone + Send + Sync,
    SrcView: IndexView<Key> + ?Sized,
    F: Fn(&Key, &SrcView::Item) -> DstItem + Send + Sync,
{
    type Msg = IndexArea<Key>;
}

impl<Key, DstItem, SrcView, F> IndexView<Key> for MapIndexItem<Key, DstItem, SrcView, F>
where
    Key: Clone + Send + Sync,
    SrcView: IndexView<Key> + ?Sized,
    F: Fn(&Key, &SrcView::Item) -> DstItem + Send + Sync,
{
    type Item = DstItem;

    fn get(&self, key: &Key) -> Option<Self::Item> {
        self.src_view
            .get(key)
            .as_ref()
            .map(|item| (self.f)(key, item))
    }

    fn area(&self) -> IndexArea<Key> {
        self.src_view.area()
    }
}

impl<Key, DstItem, SrcView, F> Observer<SrcView> for MapIndexItem<Key, DstItem, SrcView, F>
where
    Key: Clone + Send + Sync,
    SrcView: IndexView<Key> + ?Sized,
    F: Fn(&Key, &SrcView::Item) -> DstItem + Send + Sync,
{
    fn reset(&mut self, view: Option<Arc<SrcView>>) {
        let old_area = self.area();

        self.src_view = view;

        self.cast.notify(&old_area);
        self.cast.notify(&self.src_view.area())
    }

    fn notify(&mut self, area: &IndexArea<Key>) {
        self.cast.notify(area);
    }
}
