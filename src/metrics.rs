// Copyright 2014 The Prometheus Authors
// Copyright 2016 PingCAP, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cmp::{Eq, Ord, Ordering, PartialOrd};
use std::collections::HashMap;
use std::fmt::Debug;
use std::slice::Iter;

use desc::{Desc, Describer};
use errors::Result;
use proto::{self, LabelPair};

pub const SEPARATOR_BYTE: u8 = 0xFF;

pub trait Labels: Debug + Clone + Send + Sync {
    type Item: AsRef<str> + Debug + Clone + Send + Sync;
    type Owned: AsRef<[String]> + Debug + Clone + Send + Sync;

    fn as_slice(&self) -> &[Self::Item];

    fn to_owned(&self) -> Self::Owned;

    fn iter(&self) -> Iter<Self::Item> {
        self.as_slice().iter()
    }
}

impl<T: AsRef<str> + Debug + Clone + Send + Sync> Labels for [T; 0] {
    type Item = T;
    type Owned = [String; 0];

    fn as_slice(&self) -> &[T] {
        self.as_ref()
    }

    fn to_owned(&self) -> [String; 0] {
        []
    }
}

impl<T: AsRef<str> + Debug + Clone + Send + Sync> Labels for [T; 1] {
    type Item = T;
    type Owned = [String; 1];

    fn as_slice(&self) -> &[T] {
        self.as_ref()
    }

    fn to_owned(&self) -> [String; 1] {
        [self[0].as_ref().to_owned()]
    }
}

impl<T: AsRef<str> + Debug + Clone + Send + Sync> Labels for [T; 2] {
    type Item = T;
    type Owned = [String; 2];

    fn as_slice(&self) -> &[T] {
        self.as_ref()
    }

    fn to_owned(&self) -> [String; 2] {
        [self[0].as_ref().to_owned(), self[1].as_ref().to_owned()]
    }
}

impl<T: AsRef<str> + Debug + Clone + Send + Sync> Labels for [T; 3] {
    type Item = T;
    type Owned = [String; 3];

    fn as_slice(&self) -> &[T] {
        self.as_ref()
    }

    fn to_owned(&self) -> [String; 3] {
        [
            self[0].as_ref().to_owned(),
            self[1].as_ref().to_owned(),
            self[2].as_ref().to_owned(),
        ]
    }
}

impl<T: AsRef<str> + Debug + Clone + Send + Sync> Labels for [T; 4] {
    type Item = T;
    type Owned = [String; 4];

    fn as_slice(&self) -> &[T] {
        self.as_ref()
    }

    fn to_owned(&self) -> [String; 4] {
        [
            self[0].as_ref().to_owned(),
            self[1].as_ref().to_owned(),
            self[2].as_ref().to_owned(),
            self[3].as_ref().to_owned(),
        ]
    }
}

/// An interface for collecting metrics.
pub trait Collector: Sync + Send {
    /// Return descriptors for metrics.
    fn desc(&self) -> Vec<&Desc>;

    /// Collect metrics.
    fn collect(&self) -> Vec<proto::MetricFamily>;

    /// Alias of [`Registry::try_register`]. Register the collector to a registry.
    fn try_register(self, registry: &super::Registry) -> Result<Self>
    where
        Self: 'static + Sized + Clone,
    {
        registry.try_register(Box::new(self.clone())).map(|_| self)
    }

    /// Alias of [`Registry::register`]. Register the collector to a registry.
    /// Panics if there are errors.
    fn register(self, registry: &super::Registry) -> Self
    where
        Self: 'static + Sized + Clone,
    {
        registry
            .try_register(Box::new(self.clone()))
            .map(|_| self)
            .unwrap()
    }

    /// Alias of [`try_register`]. Register the collector to the default registry.
    fn try_register_default(self) -> Result<Self>
    where
        Self: 'static + Sized + Clone,
    {
        ::try_register(Box::new(self.clone())).map(|_| self)
    }

    /// Alias of [`register`]. Register the collector to the default registry.
    /// Panics if there are errors.
    fn register_default(self) -> Self
    where
        Self: 'static + Sized + Clone,
    {
        ::try_register(Box::new(self.clone()))
            .map(|_| self)
            .unwrap()
    }

    /// Alias for [`Registry::try_unregister`]. Unregister the collector from a registry.
    fn try_unregister(self, registry: &super::Registry) -> Result<Self>
    where
        Self: 'static + Sized + Clone,
    {
        registry
            .try_unregister(Box::new(self.clone()))
            .map(|_| self)
    }

    /// Alias for [`Registry::try_unregister`]. Unregister the collector from a registry.
    /// Panics if there are errors.
    fn unregister(self, registry: &super::Registry) -> Self
    where
        Self: 'static + Sized + Clone,
    {
        registry
            .try_unregister(Box::new(self.clone()))
            .map(|_| self)
            .unwrap()
    }

    /// Alias for [`try_unregister`]. Unregister the collector from the default registry.
    fn try_unregister_default(self) -> Result<Self>
    where
        Self: 'static + Sized + Clone,
    {
        ::try_unregister(Box::new(self.clone())).map(|_| self)
    }

    /// Alias for [`try_unregister`]. Unregister the collector from the default registry.
    /// Panics if there are errors.
    fn unregister_default(self) -> Self
    where
        Self: 'static + Sized + Clone,
    {
        ::try_unregister(Box::new(self.clone()))
            .map(|_| self)
            .unwrap()
    }
}

/// An interface models a single sample value with its meta data being exported to Prometheus.
pub trait Metric: Sync + Send + Clone {
    /// Return the protocol Metric.
    fn metric(&self) -> proto::Metric;
}

/// A struct that bundles the options for creating most [`Metric`](::core::Metric) types.
#[derive(Debug, Clone)]
pub struct Opts<L: Labels> {
    /// namespace, subsystem, and name are components of the fully-qualified
    /// name of the [`Metric`](::core::Metric) (created by joining these components with
    /// "_"). Only Name is mandatory, the others merely help structuring the
    /// name. Note that the fully-qualified name of the metric must be a
    /// valid Prometheus metric name.
    pub namespace: String,
    pub subsystem: String,
    pub name: String,

    /// help provides information about this metric. Mandatory!
    ///
    /// Metrics with the same fully-qualified name must have the same Help
    /// string.
    pub help: String,

    /// const_labels are used to attach fixed labels to this metric. Metrics
    /// with the same fully-qualified name must have the same label names in
    /// their ConstLabels.
    ///
    /// Note that in most cases, labels have a value that varies during the
    /// lifetime of a process. Those labels are usually managed with a metric
    /// vector collector (like CounterVec, GaugeVec). ConstLabels
    /// serve only special purposes. One is for the special case where the
    /// value of a label does not change during the lifetime of a process,
    /// e.g. if the revision of the running binary is put into a
    /// label. Another, more advanced purpose is if more than one [`Collector`](::core::Collector)
    /// needs to collect Metrics with the same fully-qualified name. In that
    /// case, those Metrics must differ in the values of their
    /// ConstLabels. See the [`Collector`](::core::Collector) examples.
    ///
    /// If the value of a label never changes (not even between binaries),
    /// that label most likely should not be a label at all (but part of the
    /// metric name).
    pub const_labels: HashMap<String, String>,

    /// variable_labels contains names of labels for which the metric maintains
    /// variable values. Metrics with the same fully-qualified name must have
    /// the same label names in their variable_labels.
    ///
    /// Note that variable_labels is used in `MetricVec`. To create a single
    /// metric must leave it empty.
    pub variable_labels: L::Owned,
}

impl Opts<[&'static str; 0]> {
    /// Creates a [`Opts`](::Opts).
    pub fn new<A, B>(name: A, help: B) -> Self
    where
        A: Into<String>,
        B: Into<String>,
    {
        Opts::new_with_label(name, help, [])
    }
}

impl<A, B> From<(A, B)> for Opts<[&'static str; 0]>
where
    A: Into<String>,
    B: Into<String>,
{
    fn from(options: (A, B)) -> Self {
        Opts::new(options.0, options.1)
    }
}

impl<L: Labels> Opts<L> {
    /// Creates a [`Opts`](::Opts) with labels.
    pub fn new_with_label<A, B>(name: A, help: B, variable_labels: L) -> Self
    where
        A: Into<String>,
        B: Into<String>,
    {
        Opts {
            namespace: "".to_owned(),
            subsystem: "".to_owned(),
            name: name.into(),
            help: help.into(),
            const_labels: HashMap::new(),
            variable_labels: variable_labels.to_owned(),
        }
    }

    /// `namespace` sets the namespace.
    pub fn namespace<S: Into<String>>(mut self, namesapce: S) -> Self {
        self.namespace = namesapce.into();
        self
    }

    /// `subsystem` sets the sub system.
    pub fn subsystem<S: Into<String>>(mut self, subsystem: S) -> Self {
        self.subsystem = subsystem.into();
        self
    }

    /// `const_labels` sets the const labels.
    pub fn const_labels(mut self, const_labels: HashMap<String, String>) -> Self {
        self.const_labels = const_labels;
        self
    }

    /// `const_label` adds a const label.
    pub fn const_label<A, B>(mut self, name: A, value: B) -> Self
    where
        A: Into<String>,
        B: Into<String>,
    {
        self.const_labels.insert(name.into(), value.into());
        self
    }

    /// `variable_labels` sets the variable labels.
    pub fn variable_labels(mut self, variable_labels: L) -> Self {
        self.variable_labels = variable_labels.to_owned();
        self
    }

    /// `fq_name` returns the fq_name.
    pub fn fq_name(&self) -> String {
        build_fq_name(&self.namespace, &self.subsystem, &self.name)
    }
}

impl<A, B, L> From<(A, B, L)> for Opts<L>
where
    A: Into<String>,
    B: Into<String>,
    L: Labels,
{
    fn from(options: (A, B, L)) -> Self {
        Opts::new_with_label(options.0, options.1, options.2)
    }
}

impl<L: Labels> Describer for Opts<L> {
    fn describe(&self) -> Result<Desc> {
        Desc::new(
            self.fq_name(),
            self.help.clone(),
            self.variable_labels.as_ref().to_vec(),
            self.const_labels.clone(),
        )
    }
}

impl Ord for LabelPair {
    fn cmp(&self, other: &LabelPair) -> Ordering {
        self.get_name().cmp(other.get_name())
    }
}

impl Eq for LabelPair {}

impl PartialOrd for LabelPair {
    fn partial_cmp(&self, other: &LabelPair) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// `build_fq_name` joins the given three name components by "_". Empty name
/// components are ignored. If the name parameter itself is empty, an empty
/// string is returned, no matter what. [`Metric`](::core::Metric) implementations included in this
/// library use this function internally to generate the fully-qualified metric
/// name from the name component in their Opts. Users of the library will only
/// need this function if they implement their own [`Metric`](::core::Metric) or instantiate a Desc
/// directly.
fn build_fq_name(namespace: &str, subsystem: &str, name: &str) -> String {
    if name.is_empty() {
        return "".to_owned();
    }

    if !namespace.is_empty() && !subsystem.is_empty() {
        return format!("{}_{}_{}", namespace, subsystem, name);
    } else if !namespace.is_empty() {
        return format!("{}_{}", namespace, name);
    } else if !subsystem.is_empty() {
        return format!("{}_{}", subsystem, name);
    }

    name.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proto::LabelPair;
    use std::cmp::{Ord, Ordering};

    fn new_label_pair(name: &str, value: &str) -> LabelPair {
        let mut l = LabelPair::new();
        l.set_name(name.to_owned());
        l.set_value(value.to_owned());
        l
    }

    #[test]
    fn test_label_cmp() {
        let tbl = vec![
            ("k1", "k2", Ordering::Less),
            ("k1", "k1", Ordering::Equal),
            ("k1", "k0", Ordering::Greater),
        ];

        for (l1, l2, order) in tbl {
            let lhs = new_label_pair(l1, l1);
            let rhs = new_label_pair(l2, l2);
            assert_eq!(lhs.cmp(&rhs), order);
        }
    }

    #[test]
    fn test_build_fq_name() {
        let tbl = vec![
            ("a", "b", "c", "a_b_c"),
            ("", "b", "c", "b_c"),
            ("a", "", "c", "a_c"),
            ("", "", "c", "c"),
            ("a", "b", "", ""),
            ("a", "", "", ""),
            ("", "b", "", ""),
            (" ", "", "", ""),
        ];

        for (namespace, subsystem, name, res) in tbl {
            assert_eq!(&build_fq_name(namespace, subsystem, name), res);
        }
    }
}
