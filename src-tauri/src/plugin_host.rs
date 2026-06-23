use rquickjs::{prelude::Func, Array, Context, Object, Runtime};
use std::time::{Duration, Instant};

const PLUGIN_TIMEOUT: Duration = Duration::from_millis(250);
const PLUGIN_MEMORY_LIMIT_BYTES: usize = 4 * 1024 * 1024;
const PLUGIN_STACK_LIMIT_BYTES: usize = 256 * 1024;
const MAX_LINES: usize = 16;

#[derive(Debug, PartialEq)]
pub struct ProviderSnapshot {
    pub provider_id: String,
    pub lines: Vec<MetricLine>,
}

#[derive(Debug, PartialEq)]
pub struct MetricLine {
    pub label: String,
    pub value: String,
}

pub trait Host {
    fn app_name(&self) -> &'static str;
}

pub struct InfUsageHost;

impl Host for InfUsageHost {
    fn app_name(&self) -> &'static str {
        "InfUsage"
    }
}

const DEMO_PROVIDER: &str = r#"
function probe(ctx) {
  return {
    providerId: "demo",
    lines: [
      { label: "Host", value: ctx.host.appName() }
    ]
  };
}
"#;

#[derive(Debug)]
pub enum PluginRunError {
    Runtime(rquickjs::Error),
    InvalidOutput(String),
}

impl From<rquickjs::Error> for PluginRunError {
    fn from(error: rquickjs::Error) -> Self {
        Self::Runtime(error)
    }
}

impl std::fmt::Display for PluginRunError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => write!(formatter, "plugin runtime error: {error}"),
            Self::InvalidOutput(message) => write!(formatter, "invalid plugin output: {message}"),
        }
    }
}

impl std::error::Error for PluginRunError {}

pub fn run_demo_provider(host: &impl Host) -> Result<ProviderSnapshot, PluginRunError> {
    run_provider(DEMO_PROVIDER, host)
}

pub fn run_provider(source: &str, host: &impl Host) -> Result<ProviderSnapshot, PluginRunError> {
    let runtime = Runtime::new()?;
    runtime.set_memory_limit(PLUGIN_MEMORY_LIMIT_BYTES);
    runtime.set_max_stack_size(PLUGIN_STACK_LIMIT_BYTES);

    let started_at = Instant::now();
    runtime.set_interrupt_handler(Some(Box::new(move || {
        started_at.elapsed() > PLUGIN_TIMEOUT
    })));

    let context = Context::full(&runtime)?;
    let app_name = host.app_name().to_string();

    context.with(|ctx| -> Result<ProviderSnapshot, PluginRunError> {
        let host = Object::new(ctx.clone())?;
        host.set("appName", Func::new(move || app_name.clone()))?;

        let plugin_context = Object::new(ctx.clone())?;
        plugin_context.set("host", host)?;
        ctx.globals().set("ctx", plugin_context)?;

        ctx.eval::<(), _>(source)?;
        let snapshot = ctx.eval::<Object, _>("probe(ctx)")?;
        let provider_id: String = snapshot.get("providerId")?;
        let lines_array = snapshot.get::<_, Array>("lines")?;

        if provider_id.trim().is_empty() {
            return Err(PluginRunError::InvalidOutput(
                "providerId must not be empty".to_string(),
            ));
        }

        if lines_array.len() > MAX_LINES {
            return Err(PluginRunError::InvalidOutput(format!(
                "provider returned more than {MAX_LINES} lines"
            )));
        }

        let lines = lines_array
            .iter::<Object>()
            .map(|line| {
                let line = line?;
                let label: String = line.get("label")?;
                let value: String = line.get("value")?;

                if label.trim().is_empty() {
                    return Err(PluginRunError::InvalidOutput(
                        "line label must not be empty".to_string(),
                    ));
                }

                Ok(MetricLine { label, value })
            })
            .collect::<Result<Vec<_>, PluginRunError>>()?;

        Ok(ProviderSnapshot { provider_id, lines })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_provider_can_only_read_through_host() {
        let snapshot = run_demo_provider(&InfUsageHost).expect("demo provider should run");

        assert_eq!(
            snapshot,
            ProviderSnapshot {
                provider_id: "demo".to_string(),
                lines: vec![MetricLine {
                    label: "Host".to_string(),
                    value: "InfUsage".to_string(),
                }],
            }
        );
    }

    #[test]
    fn rejects_empty_provider_id() {
        let error = run_provider(
            r#"
            function probe(ctx) {
              return { providerId: "", lines: [] };
            }
            "#,
            &InfUsageHost,
        )
        .expect_err("empty provider id should fail");

        assert!(matches!(error, PluginRunError::InvalidOutput(_)));
    }

    #[test]
    fn interrupts_runaway_plugin() {
        let error = run_provider(
            r#"
            function probe(ctx) {
              while (true) {}
            }
            "#,
            &InfUsageHost,
        )
        .expect_err("runaway plugin should fail");

        assert!(matches!(error, PluginRunError::Runtime(_)));
    }
}
