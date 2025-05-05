use log::LevelFilter;
use std::collections::HashMap;
pub struct LogLevelFilterFactory {
    filters: HashMap<&'static str, LevelFilter>,
    default_level: LevelFilter,
}

impl LogLevelFilterFactory {
    pub fn new() -> Self {
        Self {
            filters: HashMap::new(),
            default_level: LevelFilter::Error,
        }
    }

    pub fn get_filters(self) -> HashMap<&'static str, LevelFilter> {
        self.filters
    }

    pub fn add_filter_with_level(mut self, module: &'static str, level: LevelFilter) -> Self {
        self.filters.insert(module, level);
        self
    }

    pub fn add_filter(mut self, module: &'static str) -> Self {
        self.filters.insert(module, self.default_level);
        self
    }

    pub fn set_default_level(mut self, level: LevelFilter) -> Self {
        self.default_level = level;
        self
    }

    pub fn build(self) -> HashMap<&'static str, LevelFilter> {
        self.filters
    }
}

pub struct LogFactory {
    pub env_logger: env_logger::Builder,
}

impl Default for LogFactory {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        {
            Self::new(LevelFilter::Debug)
        }
        #[cfg(not(debug_assertions))]
        {
            Self::new(LevelFilter::Info)
        }
    }
}

impl LogFactory {
    pub fn new(level: LevelFilter) -> Self {
        let level_filters = LogLevelFilterFactory::new()
            .set_default_level(LevelFilter::Error)
            .add_filter("naga")
            .add_filter("cosmic_text")
            .add_filter("wgpu_core")
            .add_filter("wgpu_hal");
        let mut env_logger_builder = env_logger::Builder::new();
        env_logger_builder.filter_level(level);
        level_filters.get_filters().into_iter().for_each(|v| {
            env_logger_builder.filter_module(v.0, v.1);
        });

        Self {
            env_logger: env_logger_builder,
        }
    }

    pub fn custom(env_logger_builder: env_logger::Builder) -> Self {
        Self {
            env_logger: env_logger_builder,
        }
    }

    pub fn init(mut self) -> Result<(), log::SetLoggerError> {
        let logger = Logger {
            env_logger: self.env_logger.build(),
        };

        let max_level = logger.env_logger.filter();
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(max_level);
        Ok(())
    }
}

pub struct Logger {
    env_logger: env_logger::Logger,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.env_logger.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        self.env_logger.log(record)
    }

    fn flush(&self) {
        self.env_logger.flush();
    }
}
