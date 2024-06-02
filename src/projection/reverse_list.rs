use {
    crate::{
        view::{
            Observer, ObserverBroadcast, ObserverExt, OuterViewPort, View, ViewPort,
            list::{ListDiff, ListView, ListViewExt},
        },
    },
    std::sync::Arc,
    std::sync::RwLock,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Item: Clone + Send + Sync + 'static> OuterViewPort<dyn ListView<Item>> {
    pub fn reverse(&self) -> OuterViewPort<dyn ListView<Item>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));

        let map = Arc::new(RwLock::new(ReverseList {
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

pub struct ReverseList<SrcView, Item>
where Item: Clone + Send + Sync + 'static,
    SrcView: ListView<Item> + ?Sized
{
    src_view: Option<Arc<SrcView>>,
    end: usize,
    cast: Arc<RwLock<ObserverBroadcast<dyn ListView<Item>>>>
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<SrcView, Item> View for ReverseList<SrcView, Item>
where Item: Clone + Send + Sync + 'static,
    SrcView: ListView<Item> + ?Sized
{
    type Msg = ListDiff<Item>;
}

impl<SrcView, Item> ListView<Item> for ReverseList<SrcView, Item>
where Item: Clone + Send + Sync + 'static,
    SrcView: ListView<Item> + ?Sized
{
    fn len(&self) -> Option<usize> {
        Some(self.end)
    }

    fn get(&self, idx: &usize) -> Option<Item> {
        if *idx < self.end {
            self.src_view.get( &(self.end - idx - 1) )
        } else {
            None
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Item, SrcView> Observer<SrcView> for ReverseList<SrcView, Item>
where Item: Clone + Send + Sync + 'static,
    SrcView: ListView<Item> + ?Sized
{
    fn reset(&mut self, view: Option<Arc<SrcView>>) {
        self.src_view = view;

        self.cast.notify(&ListDiff::Clear);
        if let Some(v) = self.src_view.as_ref() {
            self.end = v.len().unwrap();
            for idx in 0 .. self.end {
                if idx < self.end {
                    let val = v.get( &(self.end - idx - 1) ).unwrap();
                    self.cast.notify(&ListDiff::Insert{ idx: idx, val });
                }
            }
        }
    }

    fn notify(&mut self, msg: &ListDiff<Item>) {
        /* todo optimize
         */
        //let len = self.src_view.len().unwrap();

        self.cast.notify(&match msg {
            ListDiff::Clear => {
                self.end = 0;
                ListDiff::Clear
            },
            ListDiff::Remove(mut idx) => {
                self.end -= 1;
                idx = self.end - idx;
                ListDiff::Remove(idx)
            }
            ListDiff::Insert{ mut idx, val } => {
                idx = self.end - idx;
                self.end += 1;
                ListDiff::Insert{ idx, val:val.clone() }
            }
            ListDiff::Update{ mut idx, val } => {
                idx = self.end - idx - 1;
                ListDiff::Update{ idx, val:val.clone() }
            }
        });
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[cfg(test)]
mod tests {
    use crate::buffer::vec::*;
    use crate::view::{port::UpdateTask, list::ListView};

    #[test]
    fn rev_list1() {
        let mut buffer = VecBuffer::new();

        let target_port = buffer.get_port().to_list().reverse();
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

