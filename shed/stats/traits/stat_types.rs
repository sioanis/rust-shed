/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License found in the LICENSE file in the root
 * directory of this source tree.
 */

use auto_impl::auto_impl;

pub type BoxCounter = Box<dyn Counter + Send + Sync>;
pub type BoxTimeseries = Box<dyn Timeseries + Send + Sync>;
pub type BoxHistogram = Box<dyn Histogram + Send + Sync>;

/// Counter is the simples type of stat, it behaves as a single number that can
/// be incremented.
#[auto_impl(Box)]
pub trait Counter {
    /// Increments the counter by the given amount.
    fn increment_value(&self, value: i64);
}

/// Timeseries is a type of stat that can aggregate data send to it into
/// predefined intervals of time. Example aggregations are average, sum or rate.
#[auto_impl(Box)]
pub trait Timeseries {
    /// Adds value to the timeseries. It is being aggregated based on ExportType
    fn add_value(&self, value: i64);

    /// You might want to call this method when you have a very hot counter to avoid some
    /// congestions on it.
    /// Value is the sum of values of the samples and nsamples is the number of samples.
    /// Please notice that difference in the value semantic compared to
    /// `Histogram::add_repeated_value`.
    fn add_value_aggregated(&self, value: i64, nsamples: u32);
}

/// Histogram is a type of stat that can aggregate data send to it into
/// predefined buckets. Example aggregations are average, sum or P50 (percentile).
/// The aggregation should also happen on an interval basis, since its rarely
/// useful to see aggregated all-time stats of a service running for many days.
#[auto_impl(Box)]
pub trait Histogram {
    /// Adds value to the histogram. It is being aggregated based on ExportType
    fn add_value(&self, value: i64);

    /// You might want to call this method when you have a very hot counter to avoid some
    /// congestions on it.
    /// Value is the value of a single samples and nsamples is the number of samples.
    /// Please notice that difference in the value semantic compared to
    /// `Timeseries::add_value_aggregated`.
    fn add_repeated_value(&self, value: i64, nsamples: u32);
}

mod localkey_impls {
    use super::*;
    use std::thread::LocalKey;

    pub trait CounterStatic {
        fn increment_value(&'static self, value: i64);
    }

    impl<T: Counter> CounterStatic for LocalKey<T> {
        fn increment_value(&'static self, value: i64) {
            self.with(|s| T::increment_value(s, value));
        }
    }

    pub trait TimeseriesStatic {
        fn add_value(&'static self, value: i64);
        fn add_value_aggregated(&'static self, value: i64, nsamples: u32);
    }

    impl<T: Timeseries> TimeseriesStatic for LocalKey<T> {
        fn add_value(&'static self, value: i64) {
            self.with(|s| s.add_value(value));
        }

        fn add_value_aggregated(&'static self, value: i64, nsamples: u32) {
            self.with(|s| s.add_value_aggregated(value, nsamples));
        }
    }

    pub trait HistogramStatic {
        fn add_value(&'static self, value: i64);
        fn add_repeated_value(&'static self, value: i64, nsamples: u32);
    }

    impl<T: Histogram> HistogramStatic for LocalKey<T> {
        fn add_value(&'static self, value: i64) {
            self.with(|s| s.add_value(value));
        }

        fn add_repeated_value(&'static self, value: i64, nsamples: u32) {
            self.with(|s| s.add_repeated_value(value, nsamples));
        }
    }
}
pub use localkey_impls::*;