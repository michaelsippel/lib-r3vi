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
    pub fn reverse(&self) -> OuterViewPort<dyn SequenceView<Item = Item>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));

        let map = Arc::new(RwLock::new(ReverseSequence {
            src_view: None,
            end: 0,
            cast: port.inner().get_broadcast(),
        }));

        self.add_observer(map.clone());
        port.inner().set_view(Some(map));
        port.into_outer()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct ReverseSequence<SrcView>
where SrcView: SequenceView + ?Sized
{
    src_view: Option<Arc<SrcView>>,
    end: usize,
    cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = SrcView::Item>>>>
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<SrcView> View for ReverseSequence<SrcView>
where SrcView: SequenceView + ?Sized
{
    type Msg = usize;
}

impl<SrcView> SequenceView for ReverseSequence<SrcView>
where SrcView: SequenceView + ?Sized
{
    type Item = SrcView::Item;

    fn len(&self) -> Option<usize> {
        self.src_view.len()
    }

    fn get(&self, idx: &usize) -> Option<Self::Item> {
        let len = self.src_view.len().unwrap();
        if *idx < len {
            let end = len - 1;
            self.src_view.get( &(end - idx) )
        } else {
            None
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<SrcView> Observer<SrcView> for ReverseSequence<SrcView>
where SrcView: SequenceView + ?Sized
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
            self.end = if len > 0 { len - 1 } else { 0 };
        }
    }

    fn notify(&mut self, idx: &usize) {
        /* todo optimize
         */
        let len = self.src_view.len().unwrap();
        let new_end = if len > 0 { len - 1 } else { 0 };
        if new_end < self.end {
            self.cast.notify_each( 0 ..= self.end );
            self.end = new_end;
        } else {
            self.cast.notify_each( 0 ..= new_end );
            self.end = new_end;
//            self.cast.notify( &(self.end - idx) );
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[cfg(test)]
mod tests {
    use crate::buffer::vec::*;
    use crate::projection::map_sequence::*;

    use crate::view::{port::UpdateTask, sequence::SequenceView};

    #[test]
    fn rev_seq1() {
        let mut buffer = VecBuffer::new();

        let target_port = buffer.get_port().to_sequence().reverse();
        let target_view = target_port.get_view();

        buffer.push(0);
        buffer.push(7);
        buffer.push(9);

        target_port.0.update();

        assert_eq!(target_view.len(), Some(3));

        assert_eq!(target_view.get(&0), Some(9));
        assert_eq!(target_view.get(&1), Some(7));
        assert_eq!(target_view.get(&2), Some(0));
        assert_eq!(target_view.get(&3), None);

        buffer.remove(0);

        target_port.0.update();
        assert_eq!(target_view.len(), Some(2));
        assert_eq!(target_view.get(&0), Some(9));
        assert_eq!(target_view.get(&1), Some(7));
        assert_eq!(target_view.get(&2), None);
     }
}

