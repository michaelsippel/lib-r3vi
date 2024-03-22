use {
    crate::{
        view::{
            InnerViewPort, Observer, ObserverBroadcast, ObserverExt, OuterViewPort, View, ViewPort,
            list::{ListView, ListDiff},
        },
        buffer::vec::VecDiff,
    },
    std::sync::Arc,
    std::sync::RwLock,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

/// Adapter View implementing `List` for `Vec`
pub struct Vec2List<T>
where
    T: Clone + Send + Sync + 'static,
{
    cur_len: usize,
    src_view: Option<Arc<RwLock<Vec<T>>>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn ListView<T>>>>,
}

impl<T> Vec2List<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(port: InnerViewPort<dyn ListView<T>>) -> Arc<RwLock<Self>> {
        let v2l = Arc::new(RwLock::new(Vec2List {
            cur_len: 0,
            src_view: None,
            cast: port.get_broadcast(),
        }));
        port.set_view(Some(v2l.clone()));
        v2l
    }
}

impl<T> Observer<RwLock<Vec<T>>> for Vec2List<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn reset(&mut self, view: Option<Arc<RwLock<Vec<T>>>>) {
        let old_len = self.cur_len;
        self.src_view = view;
        let new_len = if let Some(data) = self.src_view.as_ref() {
            let data = data.read().unwrap();
            self.cast.notify(&ListDiff::Clear);
            self.cast.notify_each(
                data.iter().cloned()
                    .enumerate()
                    .map(|(idx, val)|
                        ListDiff::Insert { idx, val }
                    )
            );
            data.len()
        } else {
            0
        };

        self.cur_len = new_len;
    }

    fn notify(&mut self, diff: &VecDiff<T>) {
        match diff {
            VecDiff::Clear => {
                self.cast.notify(&ListDiff::Clear);
            }
            VecDiff::Push(val) => {
                self.cast.notify(&ListDiff::Insert{
                    idx: self.cur_len,
                    val: val.clone()
                });
                self.cur_len += 1;
            }
            VecDiff::Remove(idx) => {
                self.cast.notify(&ListDiff::Remove(*idx));
                self.cur_len -= 1;
            }
            VecDiff::Insert { idx, val } => {
                self.cur_len += 1;
                self.cast.notify(&ListDiff::Insert { idx: *idx, val: val.clone() });
            }
            VecDiff::Update { idx, val } => {
                self.cast.notify(&ListDiff::Update { idx: *idx, val: val.clone() });
            }
        }
    }
}

impl<T> View for Vec2List<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Msg = ListDiff<T>;
}

impl<T> ListView<T> for Vec2List<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn get(&self, idx: &usize) -> Option<T> {
        self.src_view.as_ref()?.read().unwrap().get(*idx).cloned()
    }

    fn len(&self) -> Option<usize> {
        Some(self.cur_len)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<T> OuterViewPort<RwLock<Vec<T>>>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn to_list(&self) -> OuterViewPort<dyn ListView<T>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));

        let v2l = Vec2List::new(port.inner());
        self.add_observer(v2l.clone());
        port.into_outer()
    }
}


//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[cfg(test)]
mod tests {
    use crate::buffer::vec::VecBuffer;
    use crate::view::{
        port::UpdateTask,
        list::ListView,
    };

    #[test]
    fn vec_to_list() {
        let mut buf = VecBuffer::<char>::new();
        let list_view = buf.get_port().to_list();

        assert_eq!(list_view.get_view().len(), Some(0));

        buf.push('a');

        list_view.0.update();
        assert_eq!(list_view.get_view().len(), Some(1));
        assert_eq!(list_view.get_view().get(&0), Some('a'));
        assert_eq!(list_view.get_view().get(&1), None);

        
        buf.push('b');

        list_view.0.update();
        assert_eq!(list_view.get_view().len(), Some(2));
        assert_eq!(list_view.get_view().get(&0), Some('a'));
        assert_eq!(list_view.get_view().get(&1), Some('b'));
        assert_eq!(list_view.get_view().get(&2), None);


        buf.push('c');
        buf.remove(0);

        list_view.0.update();
        assert_eq!(list_view.get_view().len(), Some(2));
        assert_eq!(list_view.get_view().get(&0), Some('b'));
        assert_eq!(list_view.get_view().get(&1), Some('c'));
        assert_eq!(list_view.get_view().get(&2), None);
    }
}


