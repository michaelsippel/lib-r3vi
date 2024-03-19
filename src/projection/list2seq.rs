use {
    crate::{
        view::{
            InnerViewPort, Observer, ObserverBroadcast, ObserverExt, OuterViewPort, View, ViewPort,
            sequence::SequenceView,
            list::{ListView, ListDiff}
        },
        buffer::vec::VecDiff,
    },
    std::sync::Arc,
    std::sync::RwLock,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

/// Adapter View implementing `List` for `Vec`
pub struct List2Seq<T>
where
    T: Clone + Send + Sync + 'static,
{
    cur_len: usize,
    src_view: Option<Arc<dyn ListView<T>>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = T>>>>,
}

impl<T> List2Seq<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(port: InnerViewPort<dyn SequenceView<Item = T>>) -> Arc<RwLock<Self>> {
        let l2s = Arc::new(RwLock::new(List2Seq {
            cur_len: 0,
            src_view: None,
            cast: port.get_broadcast(),
        }));
        port.set_view(Some(l2s.clone()));
        l2s
    }
}

impl<T> Observer<dyn ListView<T>> for List2Seq<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn reset(&mut self, view: Option<Arc<dyn ListView<T>>>) {
        let old_len = self.cur_len;
        self.src_view = view;
        let new_len = if let Some(src_view) = self.src_view.as_ref() {
            src_view.len().unwrap_or(0)
        } else {
            0
        };

        self.cur_len = new_len;
        self.cast.notify_each(0..std::cmp::max(old_len, new_len));
    }

    fn notify(&mut self, diff: &ListDiff<T>) {
        match diff {
            ListDiff::Clear => {
                self.cast.notify_each(0..self.cur_len);
                self.cur_len = 0
            }
            ListDiff::Remove(idx) => {
                self.cast.notify_each(*idx..self.cur_len);
                self.cur_len -= 1;
            }
            ListDiff::Insert { idx, val: _ } => {
                self.cur_len += 1;
                self.cast.notify_each(*idx..self.cur_len);
            }
            ListDiff::Update { idx, val: _ } => {
                self.cast.notify(&idx);
            }
        }
    }
}

impl<T> View for List2Seq<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Msg = usize;
}

impl<T> SequenceView for List2Seq<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Item = T;

    fn get(&self, idx: &usize) -> Option<T> {
        self.src_view.as_ref()?.get(idx).clone()
    }

    fn len(&self) -> Option<usize> {
        Some(self.cur_len)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<T> OuterViewPort<dyn ListView<T>>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn to_sequence(&self) -> OuterViewPort<dyn SequenceView<Item = T>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));

        let l2s = List2Seq::new(port.inner());
        self.add_observer(l2s.clone());
        port.into_outer()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[cfg(test)]
mod tests {
    use crate::buffer::vec::VecBuffer;
    use crate::view::port::UpdateTask;

    #[test]
    fn list_to_seq() {
        let mut buf = VecBuffer::<char>::new();
        let seq_view = buf.get_port().to_list().to_sequence();

        assert_eq!(seq_view.get_view().unwrap().len(), Some(0));

        buf.push('a');

        seq_view.0.update();
        assert_eq!(seq_view.get_view().unwrap().len(), Some(1));
        assert_eq!(seq_view.get_view().unwrap().get(&0), Some('a'));
        assert_eq!(seq_view.get_view().unwrap().get(&1), None);
        
        
        buf.push('b');

        seq_view.0.update();
        assert_eq!(seq_view.get_view().unwrap().len(), Some(2));
        assert_eq!(seq_view.get_view().unwrap().get(&0), Some('a'));
        assert_eq!(seq_view.get_view().unwrap().get(&1), Some('b'));
        assert_eq!(seq_view.get_view().unwrap().get(&2), None);


        buf.push('c');
        buf.remove(0);

        seq_view.0.update();
        assert_eq!(seq_view.get_view().unwrap().len(), Some(2));
        assert_eq!(seq_view.get_view().unwrap().get(&0), Some('b'));
        assert_eq!(seq_view.get_view().unwrap().get(&1), Some('c'));
        assert_eq!(seq_view.get_view().unwrap().get(&2), None);
    }
}


