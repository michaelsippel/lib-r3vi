use {
    crate::{
        view::{
            InnerViewPort, Observer, ObserverBroadcast, ObserverExt, OuterViewPort, View, ViewPort,
            sequence::SequenceView,
            list::{ListView, ListDiff}
        },
        buffer::vec::VecDiff,
        projection::list2seq::List2Seq,
    },
    std::sync::Arc,
    std::sync::RwLock,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

/// Adapter View implementing `List` for `Vec`
pub struct Seq2List<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub(crate) cur_len: usize,
    src_view: Option<Arc<dyn SequenceView<Item = T>>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn ListView<T>>>>,
}

impl<T> Seq2List<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(port: InnerViewPort<dyn ListView<T>>) -> Arc<RwLock<Self>> {
        let s2l = Arc::new(RwLock::new(Seq2List {
            cur_len: 0,
            src_view: None,
            cast: port.get_broadcast(),
        }));
        port.set_view(Some(s2l.clone()));
        s2l
    }
}

impl<T> Observer<dyn SequenceView<Item = T>> for Seq2List<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn reset(&mut self, view: Option<Arc<dyn SequenceView<Item = T>>>) {
        let old_len = self.cur_len;
        self.src_view = view;
        let new_len = if let Some(src_view) = self.src_view.as_ref() {
            src_view.len().unwrap_or(0)
        } else {
            0
        };

        //self.cast.notify_each(0..std::cmp::max(old_len, new_len));

        self.cur_len = 0;
        self.cast.notify(&ListDiff::Clear);
        for i in 0..new_len {
            self.cast.notify(&ListDiff::Insert{ idx: i, val: self.src_view.get(&i).unwrap() });
        }
        self.cur_len = new_len;
    }

    fn notify(&mut self, idx: &usize) {
        if let Some(v) = self.src_view.get(idx) {
            if *idx < self.cur_len {
                self.cast.notify(&ListDiff::Update{ idx: *idx, val: self.src_view.get(idx).unwrap() })
            } else {
                self.cur_len += 1;
                self.cast.notify(&ListDiff::Insert{ idx: *idx, val: self.src_view.get(idx).unwrap() })
            }
        } else {
            self.cur_len -= 1;
//            assert!( *idx >= self.cur_len );
            self.cast.notify(&ListDiff::Remove(self.cur_len));
        }
    }
}

impl<T> View for Seq2List<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Msg = ListDiff<T>;
}

impl<T> ListView<T> for Seq2List<T>
where
    T: Clone + Send + Sync + 'static,
{

    fn get(&self, idx: &usize) -> Option<T> {
        self.src_view.as_ref()?.get(idx).clone()
    }

    fn len(&self) -> Option<usize> {
        Some(self.cur_len)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<T> OuterViewPort<dyn SequenceView<Item = T>>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn to_list(&self) -> OuterViewPort<dyn ListView<T>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));

        let s2l = Seq2List::new(port.inner());
        self.add_observer(s2l.clone());
        port.into_outer()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
/*
#[cfg(test)]
mod tests {
    use crate::buffer::vec::VecBuffer;
    use crate::view::{
        port::UpdateTask,
        list::ListView,
        sequence::*
    };

    #[test]
    fn vec_to_list_to_seq() {
        let mut buf = VecBuffer::<char>::new();
        let seq_view = buf.get_port().to_list().to_sequence();

        assert_eq!(seq_view.get_view().len(), Some(0));

        buf.push('a');

        seq_view.0.update();
        assert_eq!(seq_view.get_view().len(), Some(1));
        assert_eq!(seq_view.get_view().get(&0), Some('a'));
        assert_eq!(seq_view.get_view().get(&1), None);
        
        
        buf.push('b');

        seq_view.0.update();
        assert_eq!(seq_view.get_view().len(), Some(2));
        assert_eq!(seq_view.get_view().get(&0), Some('a'));
        assert_eq!(seq_view.get_view().get(&1), Some('b'));
        assert_eq!(seq_view.get_view().get(&2), None);


        buf.push('c');
        buf.remove(0);

        seq_view.0.update();
        assert_eq!(seq_view.get_view().len(), Some(2));
        assert_eq!(seq_view.get_view().get(&0), Some('b'));
        assert_eq!(seq_view.get_view().get(&1), Some('c'));
        assert_eq!(seq_view.get_view().get(&2), None);
    }
}
*/

