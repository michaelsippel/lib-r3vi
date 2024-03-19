use {
    crate::{
        view::{
            Observer, ObserverBroadcast, ObserverExt, OuterViewPort, View, ViewPort,
            list::{ListView, ListDiff}
        },
    },
    std::sync::Arc,
    std::sync::RwLock,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Item: Clone + Send + Sync + 'static> OuterViewPort<dyn ListView<Item>> {
    pub fn map<DstItem: Clone + Send + Sync + 'static, F: Fn(&Item) -> DstItem + Send + Sync + 'static>(
        &self,
        f: F,
    ) -> OuterViewPort<dyn ListView<DstItem>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));

        let map = Arc::new(RwLock::new(MapListItem {
            _phantom: std::marker::PhantomData::default(),
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

pub struct MapListItem<SrcItem, DstItem, SrcView, F>
where
    SrcItem: Clone + Send + Sync + 'static,
    DstItem: Clone + Send + Sync + 'static,
    SrcView: ListView<SrcItem> + ?Sized,
    F: Fn(&SrcItem) -> DstItem + Send + Sync,
{
    _phantom: std::marker::PhantomData< SrcItem >,
    src_view: Option<Arc<SrcView>>,
    f: F,
    cast: Arc<RwLock<ObserverBroadcast<dyn ListView<DstItem>>>>,
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<SrcItem, DstItem, SrcView, F> View for MapListItem<SrcItem, DstItem, SrcView, F>
where
    SrcItem: Clone + Send + Sync + 'static,
    DstItem: Clone + Send + Sync + 'static,
    SrcView: ListView<SrcItem> + ?Sized,
    F: Fn(&SrcItem) -> DstItem + Send + Sync,
{
    type Msg = ListDiff<DstItem>;
}

impl<SrcItem, DstItem, SrcView, F> ListView<DstItem> for MapListItem<SrcItem, DstItem, SrcView, F>
where
    SrcItem: Clone + Send + Sync + 'static,
    DstItem: Clone + Send + Sync + 'static,
    SrcView: ListView<SrcItem> + ?Sized,
    F: Fn(&SrcItem) -> DstItem + Send + Sync,
{
    fn len(&self) -> Option<usize> {
        self.src_view.len()
    }

    fn get(&self, idx: &usize) -> Option<DstItem> {
        self.src_view.get(idx).as_ref().map(|item| (self.f)(item))
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<SrcItem, DstItem, SrcView, F> Observer<SrcView> for MapListItem<SrcItem, DstItem, SrcView, F>
where
    SrcItem: Clone + Send + Sync + 'static,
    DstItem: Clone + Send + Sync + 'static,
    SrcView: ListView<SrcItem> + ?Sized,
    F: Fn(&SrcItem) -> DstItem + Send + Sync,
{
    fn reset(&mut self, view: Option<Arc<SrcView>>) {
        let old_len = self.len();
        self.src_view = view;
        let new_len = self.len();
        /* ????
        if let Some(len) = old_len {
            self.cast.notify_each(0..len);
        }
        if let Some(len) = new_len {
            self.cast.notify_each(0..len);
        }
        */
    }

    fn notify(&mut self, msg: &ListDiff<SrcItem>) {
        let forwarded_msg =
            match msg {
                ListDiff::Clear => ListDiff::Clear,
                ListDiff::Remove(idx) => ListDiff::Remove(*idx),
                ListDiff::Insert{ idx, val } =>
                    ListDiff::Insert {
                        idx: *idx,
                        val: (self.f)(val)
                    },
                ListDiff::Update{ idx, val } =>
                    ListDiff::Update{
                        idx: *idx,
                        val: (self.f)(val)
                    }
            };
        self.cast.notify(&forwarded_msg);
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[cfg(test)]
mod tests {
    use crate::buffer::vec::*;
    use crate::projection::map_sequence::*;
    use crate::view::{port::UpdateTask, list::ListView};
    
    #[test]
    fn map_list1() {
        let mut buffer = VecBuffer::new();

        let target_port = buffer.get_port()
            .to_list()
            .map(|x| x + 10);

        let target_view = target_port.get_view();

        buffer.push(0);
        buffer.push(7);
        buffer.push(9);

        target_port.0.update();

        assert_eq!(target_view.len(), Some(3));

        assert_eq!(target_view.get(&0), Some(10));
        assert_eq!(target_view.get(&1), Some(17));
        assert_eq!(target_view.get(&2), Some(19));
        assert_eq!(target_view.get(&3), None);
    }
}

