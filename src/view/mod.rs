
                    /*\
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                   View
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                    \*/
pub trait View: Send + Sync {
    /// Notification message for the observers
    type Msg: Send + Sync;
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use std::sync::{Arc, RwLock};

impl<V: View + ?Sized> View for RwLock<V> {
    type Msg = V::Msg;
}

impl<V: View + ?Sized> View for Arc<V> {
    type Msg = V::Msg;
}

impl<V: View> View for Option<V> {
    type Msg = V::Msg;
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub mod channel;
pub mod observer;
pub mod port;

pub use {
    channel::{queue_channel, set_channel, singleton_channel, ChannelReceiver, ChannelSender},
    observer::{NotifyFnObserver, Observer, ObserverBroadcast, ObserverExt, ResetFnObserver},
    port::{AnyInnerViewPort, AnyOuterViewPort, AnyViewPort, InnerViewPort, OuterViewPort, ViewPort}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub mod singleton;
pub mod sequence;
pub mod index;
pub mod grid;

