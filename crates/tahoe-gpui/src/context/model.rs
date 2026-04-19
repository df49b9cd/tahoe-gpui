//! AI model identification and pricing.

use super::usage::Usage;

/// AI model provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    /// OpenAI models (GPT-4, o3, etc.)
    OpenAi,
    /// Anthropic models (Claude)
    Anthropic,
    /// Google models (Gemini)
    Google,
    /// Ollama local models
    Ollama,
    /// Custom/unknown provider
    Custom,
}

impl Provider {
    /// Parses a provider prefix from a model ID string.
    pub fn parse(s: &str) -> Self {
        let prefix = s.split(':').next().unwrap_or(s).to_lowercase();
        match prefix.as_str() {
            "openai" => Provider::OpenAi,
            "anthropic" => Provider::Anthropic,
            "google" | "gemini" => Provider::Google,
            "ollama" => Provider::Ollama,
            _ => Provider::Custom,
        }
    }
}

/// A parsed model identifier.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelId {
    /// The provider.
    pub provider: Provider,
    /// The model name (e.g., "gpt-4", "claude-sonnet-4").
    pub model: String,
}

impl ModelId {
    /// Parses a model ID string like "openai:gpt-4" or "anthropic:claude-sonnet-4".
    pub fn parse(s: &str) -> Self {
        let (provider_str, model) = s.split_once(':').unwrap_or(("custom", s));
        Self {
            provider: Provider::parse(provider_str),
            model: model.to_string(),
        }
    }

    /// Creates a [`ModelId`] directly from a provider and model name.
    pub fn from_parts(provider: Provider, model: impl Into<String>) -> Self {
        Self {
            provider,
            model: model.into(),
        }
    }

    /// Returns a display string for this model.
    pub fn display(&self) -> String {
        match self.provider {
            Provider::OpenAi => format!("OpenAI {}", self.model),
            Provider::Anthropic => format!("Anthropic {}", self.model),
            Provider::Google => format!("Gemini {}", self.model),
            Provider::Ollama => format!("Ollama {}", self.model),
            Provider::Custom => self.model.clone(),
        }
    }
}

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Pricing information for a model (cost per 1 million tokens).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelPricing {
    /// Cost per 1M input tokens (USD).
    pub input: f64,
    /// Cost per 1M output tokens (USD).
    pub output: f64,
    /// Cost per 1M reasoning tokens (USD).
    pub reasoning: f64,
    /// Cost per 1M cached tokens (USD).
    pub cached: f64,
}

impl ModelPricing {
    /// Creates new pricing with the given rates.
    pub fn new(input: f64, output: f64, reasoning: f64, cached: f64) -> Self {
        Self {
            input,
            output,
            reasoning,
            cached,
        }
    }

    /// Calculates the cost for given usage.
    pub fn calculate_cost(&self, usage: &Usage) -> f64 {
        (usage.input as f64 / 1_000_000.0) * self.input
            + (usage.output as f64 / 1_000_000.0) * self.output
            + (usage.reasoning as f64 / 1_000_000.0) * self.reasoning
            + (usage.cached as f64 / 1_000_000.0) * self.cached
    }
}

struct PricingEntry {
    prefix: &'static str,
    model: &'static str,
    pricing: ModelPricing,
}

/// Built-in pricing table. More-specific patterns must appear before shorter
/// prefixes because matching uses `starts_with`.
const PRICING_TABLE: &[PricingEntry] = &[
    // OpenAI
    PricingEntry {
        prefix: "openai",
        model: "gpt-4o-mini",
        pricing: ModelPricing {
            input: 0.15,
            output: 0.6,
            reasoning: 0.0,
            cached: 0.0,
        },
    },
    PricingEntry {
        prefix: "openai",
        model: "gpt-4o",
        pricing: ModelPricing {
            input: 2.5,
            output: 10.0,
            reasoning: 0.0,
            cached: 0.0,
        },
    },
    PricingEntry {
        prefix: "openai",
        model: "gpt-4-turbo",
        pricing: ModelPricing {
            input: 10.0,
            output: 30.0,
            reasoning: 0.0,
            cached: 0.0,
        },
    },
    PricingEntry {
        prefix: "openai",
        model: "gpt-4.1-nano",
        pricing: ModelPricing {
            input: 0.1,
            output: 0.4,
            reasoning: 0.0,
            cached: 0.0,
        },
    },
    PricingEntry {
        prefix: "openai",
        model: "gpt-4.1-mini",
        pricing: ModelPricing {
            input: 0.4,
            output: 1.6,
            reasoning: 0.0,
            cached: 0.0,
        },
    },
    PricingEntry {
        prefix: "openai",
        model: "gpt-4.1",
        pricing: ModelPricing {
            input: 2.0,
            output: 8.0,
            reasoning: 0.0,
            cached: 0.0,
        },
    },
    PricingEntry {
        prefix: "openai",
        model: "o3-mini",
        pricing: ModelPricing {
            input: 1.1,
            output: 4.4,
            reasoning: 4.4,
            cached: 0.0,
        },
    },
    PricingEntry {
        prefix: "openai",
        model: "o4-mini",
        pricing: ModelPricing {
            input: 1.1,
            output: 4.4,
            reasoning: 4.4,
            cached: 0.0,
        },
    },
    PricingEntry {
        prefix: "openai",
        model: "o3",
        pricing: ModelPricing {
            input: 2.0,
            output: 8.0,
            reasoning: 8.0,
            cached: 0.0,
        },
    },
    PricingEntry {
        prefix: "openai",
        model: "",
        pricing: ModelPricing {
            input: 0.0,
            output: 0.0,
            reasoning: 0.0,
            cached: 0.0,
        },
    },
    // Anthropic
    PricingEntry {
        prefix: "anthropic",
        model: "claude-opus-4",
        pricing: ModelPricing {
            input: 15.0,
            output: 75.0,
            reasoning: 0.0,
            cached: 1.875,
        },
    },
    PricingEntry {
        prefix: "anthropic",
        model: "claude-sonnet-4",
        pricing: ModelPricing {
            input: 3.0,
            output: 15.0,
            reasoning: 0.0,
            cached: 0.375,
        },
    },
    PricingEntry {
        prefix: "anthropic",
        model: "claude-3.5-sonnet",
        pricing: ModelPricing {
            input: 3.0,
            output: 15.0,
            reasoning: 0.0,
            cached: 0.375,
        },
    },
    PricingEntry {
        prefix: "anthropic",
        model: "claude-3.5-haiku",
        pricing: ModelPricing {
            input: 0.8,
            output: 4.0,
            reasoning: 0.0,
            cached: 0.1,
        },
    },
    PricingEntry {
        prefix: "anthropic",
        model: "claude-3-opus",
        pricing: ModelPricing {
            input: 15.0,
            output: 75.0,
            reasoning: 0.0,
            cached: 1.875,
        },
    },
    PricingEntry {
        prefix: "anthropic",
        model: "",
        pricing: ModelPricing {
            input: 3.0,
            output: 15.0,
            reasoning: 0.0,
            cached: 0.375,
        },
    },
    // Google
    PricingEntry {
        prefix: "google",
        model: "gemini-2.5-pro",
        pricing: ModelPricing {
            input: 1.25,
            output: 10.0,
            reasoning: 0.0,
            cached: 0.315,
        },
    },
    PricingEntry {
        prefix: "google",
        model: "gemini-2.5-flash",
        pricing: ModelPricing {
            input: 0.15,
            output: 0.6,
            reasoning: 0.6,
            cached: 0.0375,
        },
    },
    PricingEntry {
        prefix: "google",
        model: "",
        pricing: ModelPricing {
            input: 0.15,
            output: 0.6,
            reasoning: 0.0,
            cached: 0.0,
        },
    },
    PricingEntry {
        prefix: "gemini",
        model: "",
        pricing: ModelPricing {
            input: 0.15,
            output: 0.6,
            reasoning: 0.0,
            cached: 0.0,
        },
    },
    // Local
    PricingEntry {
        prefix: "ollama",
        model: "",
        pricing: ModelPricing {
            input: 0.0,
            output: 0.0,
            reasoning: 0.0,
            cached: 0.0,
        },
    },
];

/// Returns the default pricing for a given model ID from the built-in table.
pub fn default_pricing(model_id: &ModelId) -> ModelPricing {
    let prefix = match model_id.provider {
        Provider::OpenAi => "openai",
        Provider::Anthropic => "anthropic",
        Provider::Google => "google",
        Provider::Ollama => "ollama",
        Provider::Custom => "",
    };

    for entry in PRICING_TABLE {
        if entry.prefix == prefix
            && (entry.model.is_empty() || model_id.model.starts_with(entry.model))
        {
            return entry.pricing;
        }
    }

    ModelPricing::new(0.0, 0.0, 0.0, 0.0)
}

#[cfg(test)]
mod tests {
    use super::{ModelId, ModelPricing, Provider, Usage, default_pricing};
    use core::prelude::v1::test;

    #[test]
    fn parse_openai() {
        let id = ModelId::parse("openai:gpt-4");
        assert_eq!(id.provider, Provider::OpenAi);
        assert_eq!(id.model, "gpt-4");
    }

    #[test]
    fn parse_anthropic() {
        let id = ModelId::parse("anthropic:claude-sonnet-4");
        assert_eq!(id.provider, Provider::Anthropic);
        assert_eq!(id.model, "claude-sonnet-4");
    }

    #[test]
    fn parse_no_colon() {
        let id = ModelId::parse("llama2");
        assert_eq!(id.provider, Provider::Custom);
        assert_eq!(id.model, "llama2");
    }

    #[test]
    fn pricing_gpt4o_mini() {
        let id = ModelId::parse("openai:gpt-4o-mini");
        let p = default_pricing(&id);
        assert_eq!(p.input, 0.15);
    }

    #[test]
    fn pricing_gpt41_mini_not_gpt41() {
        let id = ModelId::parse("openai:gpt-4.1-mini");
        let p = default_pricing(&id);
        assert_eq!(p.input, 0.4);
    }

    #[test]
    fn pricing_o3_mini_not_o3() {
        let id = ModelId::parse("openai:o3-mini");
        let p = default_pricing(&id);
        assert_eq!(p.input, 1.1);
    }

    #[test]
    fn pricing_unknown() {
        let id = ModelId::parse("custom:mystery");
        let p = default_pricing(&id);
        assert_eq!(p.input, 0.0);
    }

    #[test]
    fn calculate_cost_basic() {
        let pricing = ModelPricing::new(3.0, 15.0, 0.0, 0.375);
        let usage = Usage::new(50_000, 30_000, 0, 10_000);
        let cost = pricing.calculate_cost(&usage);
        let expected = (50_000.0 / 1e6) * 3.0 + (30_000.0 / 1e6) * 15.0 + (10_000.0 / 1e6) * 0.375;
        assert!((cost - expected).abs() < 1e-10);
    }

    #[test]
    fn from_parts_avoids_parse() {
        let id = ModelId::from_parts(Provider::OpenAi, "gpt-4");
        assert_eq!(id.provider, Provider::OpenAi);
        assert_eq!(id.model, "gpt-4");
    }
}
