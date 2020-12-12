// Copyright 2017 Lyndon Brown
//
// This file is part of the PulseAudio Rust language binding.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.
//
// Portions of documentation are copied from the LGPL 2.1+ licensed PulseAudio C headers on a
// fair-use basis, as discussed in the overall project readme (available in the git repository).

//! Daemon introspection event subscription subsystem.
//!
//! # Overview
//!
//! The application can be notified, asynchronously, whenever the internal layout of the server
//! changes. The set of facilities and operations for which notifications are generated are
//! enumerated in [`Facility`] and [`Operation`].
//!
//! The application sets the notification mask using [`Context::subscribe()`] and the callback
//! function that will be called whenever a notification occurs using
//! [`Context::set_subscribe_callback()`].
//!
//! The mask provided to [`Context::subscribe()`] can be created by binary ORing a set of values,
//! either produced with [`Facility::to_interest_mask()`], or more simply with the provided
//! constants in the [`subscription_masks`] submodule.
//!
//! The callback will be called with event type information representing the event that caused the
//! callback, detailing *facility* and *operation*, where for instance [`Facility::Source`] with
//! [`Operation::New`] indicates that a new source was added.
//!
//! # Example
//!
//! Subscribe (declare interest):
//!
//! ```rust,ignore
//! use libpulse_binding::context::subscribe::subscription_masks;
//!
//! let interest = subscription_masks::SINK |
//!     subscription_masks::SOURCE;
//!
//! let op = my_context.subscribe(
//!     interest,   // Our interest mask
//!     |_| {}      // We won’t bother doing anything in the success callback in this example
//! );
//! ```
//!
//! [`Facility`]: enum.Facility.html
//! [`Operation`]: enum.Operation.html
//! [`Facility::Source`]: enum.Facility.html#variant.Source
//! [`Operation::New`]: enum.Operation.html#variant.New
//! [`Facility::to_interest_mask()`]: enum.Facility.html#method.to_interest_mask
//! [`Context::subscribe()`]: ../struct.Context.html#method.subscribe
//! [`Context::set_subscribe_callback()`]: ../struct.Context.html#method.set_subscribe_callback
//! [`subscription_masks`]: subscription_masks/index.html

use std::os::raw::c_void;
use bitflags::bitflags;
use num_derive::{FromPrimitive, ToPrimitive};
use super::{ContextInternal, Context};
use crate::operation;
use crate::callbacks::{box_closure_get_capi_ptr, MultiUseCallback};

pub use capi::context::subscribe::pa_subscription_event_type_t as EventType;

/// Mask to extract facility value from the event type passed to the user callback.
#[deprecated(note="use the associated constant on `Facility` instead")]
pub const FACILITY_MASK: u32 = capi::PA_SUBSCRIPTION_EVENT_FACILITY_MASK;
/// Mask to extract operation value from the event type passed to the user callback.
#[deprecated(note="use the associated constant on `Operation` instead")]
pub const OPERATION_MASK: u32 = capi::PA_SUBSCRIPTION_EVENT_TYPE_MASK;

bitflags! {
    /// A set of facility masks, to be passed to [`Context::subscribe()`].
    ///
    /// Note that you can convert a [`Facility`] to a mask with [`Facility::to_interest_mask()`].
    ///
    /// [`Context::subscribe()`]: ../struct.Context.html#method.subscribe
    /// [`Facility`]: enum.Facility.html
    /// [`Facility::to_interest_mask()`]: enum.Facility.html#method.to_interest_mask
    #[repr(transparent)]
    pub struct InterestMaskSet: u32 {
        /// No facility (null selection; zero).
        const NULL = capi::PA_SUBSCRIPTION_MASK_NULL;
        /// Sink facility.
        const SINK = capi::PA_SUBSCRIPTION_MASK_SINK;
        /// Source facility.
        const SOURCE = capi::PA_SUBSCRIPTION_MASK_SOURCE;
        /// Sink input facility.
        const SINK_INPUT = capi::PA_SUBSCRIPTION_MASK_SINK_INPUT;
        /// Source output facility.
        const SOURCE_OUTPUT = capi::PA_SUBSCRIPTION_MASK_SOURCE_OUTPUT;
        /// Module facility.
        const MODULE = capi::PA_SUBSCRIPTION_MASK_MODULE;
        /// Client facility.
        const CLIENT = capi::PA_SUBSCRIPTION_MASK_CLIENT;
        /// Sample cache facility.
        const SAMPLE_CACHE = capi::PA_SUBSCRIPTION_MASK_SAMPLE_CACHE;
        /// Server facility.
        const SERVER = capi::PA_SUBSCRIPTION_MASK_SERVER;
        /// Card facility.
        const CARD = capi::PA_SUBSCRIPTION_MASK_CARD;
        /// All facilities.
        const ALL = capi::PA_SUBSCRIPTION_MASK_ALL;
    }
}

/// A set of masks used for expressing which facilities you are interested in when subscribing.
#[deprecated(note = "Use the associated constants on `InterestMaskSet`.")]
#[allow(missing_docs)]
pub mod subscription_masks {
    use super::InterestMaskSet;

    pub const NULL:          InterestMaskSet = InterestMaskSet::NULL;
    pub const SINK:          InterestMaskSet = InterestMaskSet::SINK;
    pub const SOURCE:        InterestMaskSet = InterestMaskSet::SOURCE;
    pub const SINK_INPUT:    InterestMaskSet = InterestMaskSet::SINK_INPUT;
    pub const SOURCE_OUTPUT: InterestMaskSet = InterestMaskSet::SOURCE_OUTPUT;
    pub const MODULE:        InterestMaskSet = InterestMaskSet::MODULE;
    pub const CLIENT:        InterestMaskSet = InterestMaskSet::CLIENT;
    pub const SAMPLE_CACHE:  InterestMaskSet = InterestMaskSet::SAMPLE_CACHE;
    pub const SERVER:        InterestMaskSet = InterestMaskSet::SERVER;
    pub const MASK_CARD:     InterestMaskSet = InterestMaskSet::CARD;
    pub const ALL:           InterestMaskSet = InterestMaskSet::ALL;
}

/// Facility component of an event.
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[derive(FromPrimitive, ToPrimitive)]
#[allow(missing_docs)]
pub enum Facility {
    Sink         = 0,
    Source       = 1,
    SinkInput    = 2,
    SourceOutput = 3,
    Module       = 4,
    Client       = 5,
    SampleCache  = 6,
    /// Global server change, only occurring with
    /// [`Operation::Changed`](enum.Operation.html#variant.Changed).
    Server       = 7,
    /* NOTE: value `8` previously assigned, obsoleted */
    Card         = 9,
}

/// Operation component of an event.
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[derive(FromPrimitive, ToPrimitive)]
pub enum Operation {
    /// A new object was created
    New     = 0,
    /// A property of the object was modified
    Changed = 0x10,
    /// An object was removed
    Removed = 0x20,
}

impl Facility {
    /// Mask to extract facility value from the event type passed to the user callback.
    pub const MASK: u32 = capi::PA_SUBSCRIPTION_EVENT_FACILITY_MASK;

    fn from_int(value: u32) -> Option<Facility> {
        match value {
            0 => Some(Facility::Sink),
            1 => Some(Facility::Source),
            2 => Some(Facility::SinkInput),
            3 => Some(Facility::SourceOutput),
            4 => Some(Facility::Module),
            5 => Some(Facility::Client),
            6 => Some(Facility::SampleCache),
            7 => Some(Facility::Server),
            /* NOTE: value `8` previously assigned, obsoleted */
            9 => Some(Facility::Card),
            _ => None,
        }
    }

    /// Converts to an interest mask.
    #[inline(always)]
    pub const fn to_interest_mask(self) -> InterestMaskSet {
        InterestMaskSet::from_bits_truncate(1u32 << (self as u32))
    }
}

impl Operation {
    /// Mask to extract operation value from the event type passed to the user callback.
    pub const MASK: u32 = capi::PA_SUBSCRIPTION_EVENT_TYPE_MASK;

    fn from_int(value: u32) -> Option<Operation> {
        match value {
            0 => Some(Operation::New),
            0x10 => Some(Operation::Changed),
            0x20 => Some(Operation::Removed),
            _ => None,
        }
    }
}

/// Extracts facility from `EventType` value.
#[inline]
fn get_facility(value: EventType) -> Option<Facility> {
    Facility::from_int((value & Facility::MASK) as u32)
}

/// Extracts operation from `EventType` value.
#[inline]
fn get_operation(value: EventType) -> Option<Operation> {
    Operation::from_int((value & Operation::MASK) as u32)
}

pub(super) type Callback = MultiUseCallback<dyn FnMut(Option<Facility>, Option<Operation>, u32),
    extern "C" fn(*mut ContextInternal, EventType, u32, *mut c_void)>;

impl Context {
    /// Enables event notification.
    ///
    /// The `mask` parameter is used to specify which facilities you are interested in being
    /// modified about. Use [`set_subscribe_callback()`](#method.set_subscribe_callback) to set the
    /// actual callback that will be called when an event occurs.
    ///
    /// The callback must accept a `bool`, which indicates success.
    ///
    /// Panics if the underlying C function returns a null pointer.
    pub fn subscribe<F>(&mut self, mask: InterestMaskSet, callback: F)
        -> operation::Operation<dyn FnMut(bool)>
        where F: FnMut(bool) + 'static
    {
        let cb_data = box_closure_get_capi_ptr::<dyn FnMut(bool)>(Box::new(callback));
        let ptr = unsafe { capi::pa_context_subscribe(self.ptr, mask.bits(),
            Some(super::success_cb_proxy), cb_data) };
        assert!(!ptr.is_null());
        operation::Operation::from_raw(ptr, cb_data as *mut Box<dyn FnMut(bool)>)
    }

    /// Sets the context specific call back function that is called whenever a subscribed-to event
    /// occurs.
    ///
    /// Use [`subscribe()`](#method.subscribe) to set the facilities you are interested in receiving
    /// notifications for, and thus to start receiving notifications with the callback set here.
    ///
    /// The callback must take three parameters. The first two are the facility and operation
    /// components of the event type respectively (the underlying C API provides this information
    /// combined into a single integer, here we extract the two component parts for you); these are
    /// wrapped in `Option` wrappers should the given values ever not map to the enum variants, but
    /// it’s probably safe to always just `unwrap()` them). The third parameter is an associated
    /// index value.
    pub fn set_subscribe_callback(&mut self,
        callback: Option<Box<dyn FnMut(Option<Facility>, Option<Operation>, u32) + 'static>>)
    {
        let saved = &mut self.cb_ptrs.subscribe;
        *saved = Callback::new(callback);
        let (cb_fn, cb_data) = saved.get_capi_params(cb_proxy);
        unsafe { capi::pa_context_set_subscribe_callback(self.ptr, cb_fn, cb_data); }
    }
}

/// Proxy for callbacks.
///
/// Warning: This is for multi-use cases! It does **not** destroy the actual closure callback, which
/// must be accomplished separately to avoid a memory leak.
extern "C"
fn cb_proxy(_: *mut ContextInternal, et: EventType, index: u32, userdata: *mut c_void) {
    let _ = std::panic::catch_unwind(|| {
        let facility = get_facility(et);
        let operation = get_operation(et);
        let callback = Callback::get_callback(userdata);
        (callback)(facility, operation, index);
    });
}
