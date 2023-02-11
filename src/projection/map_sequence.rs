use {
    crate::{
        view::{
            Observer, ObserverBroadcast, ObserverExt, OuterViewPort, View, ViewPort,
            sequence::SequenceView,
        },
    },
    std::sync::Arc,
    std::sync::RwLock,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Item: 'static> OuterViewPort<dyn SequenceView<Item = Item>> {
    pub fn map<DstItem: 'static, F: Fn(&Item) -> DstItem + Send + Sync + 'static>(
        &self,
        f: F,
    ) -> OuterViewPort<dyn SequenceView<Item = DstItem>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));

        let map = Arc::new(RwLock::new(MapSequenceItem {
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

pub struct MapSequenceItem<DstItem, SrcView, F>
where
    SrcView: SequenceView + ?Sized,
    F: Fn(&SrcView::Item) -> DstItem + Send + Sync,
{
    src_view: Option<Arc<SrcView>>,
    f: F,
    cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = DstItem>>>>,
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<DstItem, SrcView, F> View for MapSequenceItem<DstItem, SrcView, F>
where
    SrcView: SequenceView + ?Sized,
    F: Fn(&SrcView::Item) -> DstItem + Send + Sync,
{
    type Msg = usize;
}

impl<DstItem, SrcView, F> SequenceView for MapSequenceItem<DstItem, SrcView, F>
where
    SrcView: SequenceView + ?Sized,
    F: Fn(&SrcView::Item) -> DstItem + Send + Sync,
{
    type Item = DstItem;

    fn len(&self) -> Option<usize> {
        self.src_view.len()
    }

    fn get(&self, idx: &usize) -> Option<DstItem> {
        self.src_view.get(idx).as_ref().map(|item| (self.f)(item))
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<DstItem, SrcView, F> Observer<SrcView> for MapSequenceItem<DstItem, SrcView, F>
where
    SrcView: SequenceView + ?Sized,
    F: Fn(&SrcView::Item) -> DstItem + Send + Sync,
{
    fn reset(&mut self, view: Option<Arc<SrcView>>) {
        let old_len = self.len();
        self.src_view = view;
        let new_len = self.len();

        if let Some(len) = old_len {
            self.cast.notify_each(0..len);
        }
        if let Some(len) = new_len {
            self.cast.notify_each(0..len);
        }
    }

    fn notify(&mut self, msg: &usize) {
        self.cast.notify(msg);
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[cfg(test)]
mod tests {
    use crate::buffer::vec::*;
    use crate::projection::map_sequence::*;

    use crate::view::port::UpdateTask;
    
    #[test]
    fn map_seq1() {
        let mut buffer = VecBuffer::new();

        let target_port = buffer.get_port().to_sequence().map(|x| x + 10);
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

