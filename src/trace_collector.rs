/*
 *  Copyright 2026 Michael Bachmann
 *
 * Licensed under either the MIT or the Apache License, Version 2.0,
 * as per the user's preference.
 * You may not use this file except in compliance with at least one
 * of these two licenses.
 * You may obtain a copy of the Licenses at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *     and
 *     https://opensource.org/license/MIT
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::fmt::Debug;
use tokio::sync::mpsc;
use tracing::{Event, Metadata, Subscriber, field::Visit};
use tracing_subscriber::{
    Layer,
    layer::{Context, Filter},
};

pub const TRACE_ONLY: TraceOnlyFilter = TraceOnlyFilter;

pub struct TraceOnlyFilter;

impl<S: Subscriber> Filter<S> for TraceOnlyFilter {
    fn enabled(&self, meta: &Metadata<'_>, _: &Context<'_, S>) -> bool {
        meta.level() == &tracing::Level::TRACE
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct FieldConsumer<'a, Str: AsRef<str>> {
    field_names: Option<&'a Box<[Str]>>,
    field_tx: &'a mpsc::Sender<Field>,
}

#[derive(Debug)]
pub struct TraceCollector {
    field_tx: mpsc::Sender<Field>,
    field_names: Option<Box<[String]>>,
}

impl TraceCollector {
    pub fn new(buffer_size: usize) -> (Self, mpsc::Receiver<Field>) {
        let (tx, rx) = mpsc::channel(buffer_size);

        let layer = Self {
            field_tx: tx,
            field_names: None,
        };

        (layer, rx)
    }

    pub fn for_fields<Str: Into<String>>(
        fields: impl IntoIterator<Item = Str>,
        buffer_size: usize,
    ) -> (Self, mpsc::Receiver<Field>) {
        let (tx, rx) = mpsc::channel(buffer_size);

        let layer = Self {
            field_tx: tx,
            field_names: Some(fields.into_iter().map(Into::into).collect()),
        };

        (layer, rx)
    }
}

impl<S: Subscriber> Layer<S> for TraceCollector {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut consumer = FieldConsumer {
            field_names: self.field_names.as_ref(),
            field_tx: &self.field_tx,
        };
        event.record(&mut consumer);
    }
}

impl<'a, Str: AsRef<str>> Visit for FieldConsumer<'a, Str> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn Debug) {
        if let Some(field_names) = &self.field_names {
            if !field_names.iter().any(|name| name.as_ref() == field.name()) {
                return;
            }
        }
        if let Err(e) = self.field_tx.try_send(Field {
            name: field.name().to_string(),
            value: format!("{value:?}"),
        }) {
            eprintln!("Failed to send field: {e}");
        }
    }
}
