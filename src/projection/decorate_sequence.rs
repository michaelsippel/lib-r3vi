use {
    crate::{
        view::{
            View, OuterViewPort, Observer, ViewPort, ObserverBroadcast,
            sequence::*
        },
        projection::projection_helper::ProjectionHelper,
    },
    std::sync::Arc,
    std::sync::RwLock,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
//                   Wrap
//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
    
pub struct Wrapped<T>
where T: Send + Sync + 'static
{
    pub(super) opening: T,
    pub(super) closing: T,
    pub(super) items: Arc<dyn SequenceView<Item = T>>,

    pub(super) cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = T>>>>,
    pub(super) proj_helper: ProjectionHelper<(), Self>,
}

impl<T> View for Wrapped<T>
where T: Clone + Send + Sync + 'static
{
    type Msg = usize;
}

impl<T> SequenceView for Wrapped<T>
where T: Clone + Send + Sync + 'static
{
    type Item = T;

    fn len(&self) -> Option<usize> {
        Some(self.items.len()? + 2)
    }

    fn get(&self, idx: &usize) -> Option<Self::Item> {
        let l = self.items.len().unwrap_or((-2 as i32) as usize);
        if *idx < l+2 {
        Some(
            if *idx == 0 {
                self.opening.clone()
            } else if *idx < l+1 {
                self.items.get(&(*idx - 1))?
            } else {
                self.closing.clone()
            })
        } else {
            None
        }
    }        
}


pub trait Wrap<T> {
    fn wrap(&self, opening: T, closing: T) -> OuterViewPort<dyn SequenceView<Item = T>>;
}

impl<T> Wrap<T> for OuterViewPort<dyn SequenceView<Item = T>>
where T: Clone + Send + Sync + 'static
{
    fn wrap(&self, opening: T, closing: T) -> OuterViewPort<dyn SequenceView<Item = T>> {
        let port = ViewPort::new();

        let mut proj_helper = ProjectionHelper::new(port.update_hooks.clone());
        let w = Arc::new(RwLock::new(Wrapped {
            opening,
            closing,
            items: proj_helper.new_sequence_arg((), self.clone(), |s: &mut Wrapped<T>, item_idx| {
                s.cast.notify(&(*item_idx + 1));
                s.cast.notify(&(*item_idx + 2));
            }),
            cast: port.get_cast(),
            proj_helper,
        }));

        w.write().unwrap().proj_helper.set_proj(&w);
        port.set_view(Some(w.clone()));
        port.into_outer()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
//                   Separate
//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct Separated<T>
where T: Send + Sync + 'static
{
    pub(super) delimiter: T,
    pub(super) items: Arc<dyn SequenceView<Item = T>>,

    pub(super) cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = T>>>>,
    pub(super) proj_helper: ProjectionHelper<(), Self>,
}

impl<T> View for Separated<T>
where T: Clone + Send + Sync + 'static    
{
    type Msg = usize;
}

impl<T> SequenceView for Separated<T>
where T: Clone + Send + Sync + 'static
{
    type Item = T;

    fn len(&self) -> Option<usize> {
        let l = self.items.len()?;
        if l == 0 {
            Some(0)
        } else if l == 1 {
            Some(1)
        } else {
            Some(l*2 - 1)
        }
    }

    fn get(&self, idx: &usize) -> Option<T> {
        let l = self.items.len().unwrap_or(usize::MAX);
        if *idx+1 < l*2 {
            if *idx % 2 == 0 {
                self.items.get(&(*idx / 2))
            } else {
                Some(self.delimiter.clone())
            }
        } else {
            None
        }
    }
}

pub trait Separate<T> {
    fn separate(&self, delimiter: T) -> OuterViewPort<dyn SequenceView<Item = T>>;
}

impl<T> Separate<T> for OuterViewPort<dyn SequenceView<Item = T>>
where T: Clone + Send + Sync + 'static
{
    fn separate(&self, delimiter: T) -> OuterViewPort<dyn SequenceView<Item = T>> {
        let port = ViewPort::new();

        let mut proj_helper = ProjectionHelper::new(port.update_hooks.clone());
        let w = Arc::new(RwLock::new(Separated {
            delimiter,
            items: proj_helper.new_sequence_arg(
                (),
                self.clone(),
                |s: &mut Separated<T>, item_idx| {
                    s.cast.notify(&(*item_idx * 2));
                    if *item_idx > 0 {
                        s.cast.notify(&(*item_idx * 2 - 1));
                    }
                }),
                    
            cast: port.get_cast(),
            proj_helper,
        }));

        w.write().unwrap().proj_helper.set_proj(&w);
        port.set_view(Some(w.clone()));
        port.into_outer()
    }
}

