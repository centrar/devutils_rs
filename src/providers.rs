//! Multi-Provider AI Support - OpenAI, Anthropic, Google, NVIDIA, and more

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

static NVIDIA_API_KEYS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "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".to_string(),
        "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".to_string(),
        "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".to_string(),
        "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".to_string(),
    ]
});

static CURRENT_KEY_INDEX: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

pub fn get_nvidia_api_key() -> String {
    let mut idx = CURRENT_KEY_INDEX.lock().unwrap();
    *idx = (*idx + 1) % NVIDIA_API_KEYS.len();
    NVIDIA_API_KEYS[*idx].clone()
}

static PROVIDERS: Lazy<Mutex<HashMap<String, ProviderConfig>>> =
    Lazy::new(|| Mutex::new(default_providers()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_key_env: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: usize,
    pub enabled: bool,
    pub priority: u32,
}

impl ProviderConfig {
    pub fn new(name: &str, api_key_env: &str, base_url: &str, model: &str) -> Self {
        Self {
            name: name.to_string(),
            api_key_env: api_key_env.to_string(),
            base_url: base_url.to_string(),
            model: model.to_string(),
            max_tokens: 4096,
            enabled: true,
            priority: 0,
        }
    }
}

fn default_providers() -> HashMap<String, ProviderConfig> {
    let mut providers = HashMap::new();

    providers.insert(
        "openai".to_string(),
        ProviderConfig::new(
            "openai",
            "OPENAI_API_KEY",
            "https://api.openai.com/v1",
            "gpt-4",
        ),
    );

    providers.insert(
        "anthropic".to_string(),
        ProviderConfig::new(
            "anthropic",
            "ANTHROPIC_API_KEY",
            "https://api.anthropic.com/v1",
            "claude-sonnet-4-5",
        ),
    );

    providers.insert(
        "google".to_string(),
        ProviderConfig::new(
            "google",
            "GOOGLE_API_KEY",
            "https://generativelanguage.googleapis.com/v1",
            "gemini-2.0-flash",
        ),
    );

    providers.insert(
        "deepseek".to_string(),
        ProviderConfig::new(
            "deepseek",
            "DEEPSEEK_API_KEY",
            "https://api.deepseek.com/v1",
            "deepseek-chat",
        ),
    );

    // Local Ollama provider
    providers.insert(
        "ollama".to_string(),
        ProviderConfig::new(
            "ollama",
            "OLLAMA_HOST",
            "http://localhost:11434/v1",
            "llama3.2",
        ),
    );

    providers.insert(
        "ollama-local".to_string(),
        ProviderConfig::new(
            "ollama-local",
            "OLLAMA_HOST",
            "http://localhost:11434",
            "llama3.2:3b",
        ),
    );

    providers.insert(
        "xai".to_string(),
        ProviderConfig::new("xai", "XAI_API_KEY", "https://api.xai.com/v1", "grok-2"),
    );

    providers.insert(
        "local".to_string(),
        ProviderConfig::new(
            "local",
            "LOCAL_API_KEY",
            "http://localhost:11434/v1",
            "llama3",
        ),
    );

    providers.insert(
        "nvidia".to_string(),
        ProviderConfig::new(
            "nvidia",
            "NVIDIA_API_KEY",
            "https://integrate.api.nvidia.com/v1",
            "meta/llama-3.1-8b-instruct",
        ),
    );

    providers.insert(
        "nvidia_r1".to_string(),
        ProviderConfig::new(
            "nvidia_r1",
            "NVIDIA_API_KEY",
            "https://integrate.api.nvidia.com/v1",
            "meta/llama-3.1-70b-instruct",
        ),
    );

    providers.insert(
        "nvidia_m2".to_string(),
        ProviderConfig::new(
            "nvidia_m2",
            "NVIDIA_API_KEY",
            "https://integrate.api.nvidia.com/v1",
            "meta/llama-3.3-70b-instruct",
        ),
    );

    providers.insert(
        "nvidia_kimi".to_string(),
        ProviderConfig::new(
            "nvidia_kimi",
            "NVIDIA_API_KEY",
            "https://integrate.api.nvidia.com/v1",
            "qwen/qwen2.5-coder-32b-instruct",
        ),
    );

    providers.insert(
        "nvidia_r1".to_string(),
        ProviderConfig::new(
            "nvidia_r1",
            "NVIDIA_API_KEY",
            "https://integrate.api.nvidia.com/v1",
            "deepseek-ai/deepseek-r1",
        ),
    );

    providers.insert(
        "nvidia_m2".to_string(),
        ProviderConfig::new(
            "nvidia_m2",
            "NVIDIA_API_KEY",
            "https://integrate.api.nvidia.com/v1",
            "minimax/MiniMax-M2.7",
        ),
    );

    providers.insert(
        "nvidia_kimi".to_string(),
        ProviderConfig::new(
            "nvidia_kimi",
            "NVIDIA_API_KEY",
            "https://integrate.api.nvidia.com/v1",
            "moonshotai/kimi-k2.5",
        ),
    );

    // === OpenRouter Providers (30+) ===
    providers.insert(
        "openrouter/anthropic".to_string(),
        ProviderConfig::new(
            "openrouter/anthropic",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "anthropic/claude-3.5-sonnet",
        ),
    );
    providers.insert(
        "openrouter/openai".to_string(),
        ProviderConfig::new(
            "openrouter/openai",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "openai/gpt-4o",
        ),
    );
    providers.insert(
        "openrouter/google".to_string(),
        ProviderConfig::new(
            "openrouter/google",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "google/gemini-pro-1.5",
        ),
    );
    providers.insert(
        "openrouter/deepseek".to_string(),
        ProviderConfig::new(
            "openrouter/deepseek",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "deepseek/deepseek-chat",
        ),
    );
    providers.insert(
        "openrouter/meta".to_string(),
        ProviderConfig::new(
            "openrouter/meta",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "meta-llama/llama-3-70b-instruct",
        ),
    );
    providers.insert(
        "openrouter/mistral".to_string(),
        ProviderConfig::new(
            "openrouter/mistral",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "mistralai/mistral-7b-instruct",
        ),
    );
    providers.insert(
        "openrouter/codellama".to_string(),
        ProviderConfig::new(
            "openrouter/codellama",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "codellama/codellama-70b",
        ),
    );
    providers.insert(
        "openrouter/mixtral".to_string(),
        ProviderConfig::new(
            "openrouter/mixtral",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "mistralai/mixtral-8x7b-instruct",
        ),
    );
    providers.insert(
        "openrouter/qwen".to_string(),
        ProviderConfig::new(
            "openrouter/qwen",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "qwen/qwen-2-72b",
        ),
    );
    providers.insert(
        "openrouter/yi".to_string(),
        ProviderConfig::new(
            "openrouter/yi",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "01-ai/yi-large",
        ),
    );
    providers.insert(
        "openrouter/grok".to_string(),
        ProviderConfig::new(
            "openrouter/grok",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "x-ai/grok-2",
        ),
    );
    providers.insert(
        "openrouter/llava".to_string(),
        ProviderConfig::new(
            "openrouter/llava",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "liuhaotian/llava-yi-34b",
        ),
    );
    providers.insert(
        "openrouter/phi".to_string(),
        ProviderConfig::new(
            "openrouter/phi",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "microsoft/phi-3-medium",
        ),
    );
    providers.insert(
        "openrouter/command".to_string(),
        ProviderConfig::new(
            "openrouter/command",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "cohere/command-r-plus",
        ),
    );
    providers.insert(
        "openrouter/wizardlm".to_string(),
        ProviderConfig::new(
            "openrouter/wizardlm",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "wizardlm/wizardlm-2-8x22b",
        ),
    );
    providers.insert(
        "openrouter/dbrx".to_string(),
        ProviderConfig::new(
            "openrouter/dbrx",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "databricks/dbrx-instruct",
        ),
    );
    providers.insert(
        "openrouter/solar".to_string(),
        ProviderConfig::new(
            "openrouter/solar",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "upstage/solar-10b-instruct",
        ),
    );
    providers.insert(
        "openrouter/nousresearch".to_string(),
        ProviderConfig::new(
            "openrouter/nousresearch",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "nousresearch/hermes-3-70b",
        ),
    );
    providers.insert(
        "openrouter/neversink".to_string(),
        ProviderConfig::new(
            "openrouter/neversink",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "neversink/llama-3-8b-instruct",
        ),
    );
    providers.insert(
        "openrouter/kimi".to_string(),
        ProviderConfig::new(
            "openrouter/kimi",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "moonshotai/kimi-chat",
        ),
    );
    providers.insert(
        "openrouter/minimax".to_string(),
        ProviderConfig::new(
            "openrouter/minimax",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "minimaxai/minimax-20b",
        ),
    );
    providers.insert(
        "openrouter/zhipu".to_string(),
        ProviderConfig::new(
            "openrouter/zhipu",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "zhipuai/glm-4-9b-chat",
        ),
    );
    providers.insert(
        "openrouter/blue".to_string(),
        ProviderConfig::new(
            "openrouter/blue",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "bluewhale/llama-3-70b",
        ),
    );
    providers.insert(
        "openrouter/reason".to_string(),
        ProviderConfig::new(
            "openrouter/reason",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "nvidia/llama-3.1-nemotron-70b-instruct",
        ),
    );
    providers.insert(
        "openrouter/mistral-nemo".to_string(),
        ProviderConfig::new(
            "openrouter/mistral-nemo",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "mistralai/mistral-nemo-instruct-2407",
        ),
    );
    providers.insert(
        "openrouter/mistral-small".to_string(),
        ProviderConfig::new(
            "openrouter/mistral-small",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "mistralai/mistral-small-instruct-2409",
        ),
    );
    providers.insert(
        "openrouter/codestral".to_string(),
        ProviderConfig::new(
            "openrouter/codestral",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "mistralai/codestral-22b-instruct",
        ),
    );
    providers.insert(
        "openrouter/deepseek-v3".to_string(),
        ProviderConfig::new(
            "openrouter/deepseek-v3",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "deepseek/deepseek-chat-v3",
        ),
    );
    providers.insert(
        "openrouter/glm4".to_string(),
        ProviderConfig::new(
            "openrouter/glm4",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "zhipuai/glm-4-plus",
        ),
    );
    providers.insert(
        "openrouter/llama-3.2".to_string(),
        ProviderConfig::new(
            "openrouter/llama-3.2",
            "OPENROUTER_API_KEY",
            "https://openrouter.ai/api/v1",
            "meta-llama/llama-3.2-90b-vision-instruct",
        ),
    );

    // === AWS Bedrock ===
    providers.insert(
        "bedrock/anthropic".to_string(),
        ProviderConfig::new(
            "bedrock/anthropic",
            "AWS_REGION",
            "https://bedrock-runtime.us-east-1.amazonaws.com",
            "anthropic.claude-3-5-sonnet-20241022-v1:0",
        ),
    );
    providers.insert(
        "bedrock/llama".to_string(),
        ProviderConfig::new(
            "bedrock/llama",
            "AWS_REGION",
            "https://bedrock-runtime.us-east-1.amazonaws.com",
            "meta.llama3-1-70b-instruct-v1:0",
        ),
    );
    providers.insert(
        "bedrock/mistral".to_string(),
        ProviderConfig::new(
            "bedrock/mistral",
            "AWS_REGION",
            "https://bedrock-runtime.us-east-1.amazonaws.com",
            "mistral.mistral-large-2407-v1:0",
        ),
    );

    // === Azure ===
    providers.insert(
        "azure/openai".to_string(),
        ProviderConfig::new(
            "azure/openai",
            "AZURE_OPENAI_KEY",
            "https://openai.openai.azure.com",
            "gpt-4o",
        ),
    );
    providers.insert(
        "azure/gpt4".to_string(),
        ProviderConfig::new(
            "azure/gpt4",
            "AZURE_OPENAI_KEY",
            "https://openai.openai.azure.com",
            "gpt-4",
        ),
    );
    providers.insert(
        "azure/gpt35".to_string(),
        ProviderConfig::new(
            "azure/gpt35",
            "AZURE_OPENAI_KEY",
            "https://openai.openai.azure.com",
            "gpt-35-turbo",
        ),
    );

    // === Google Vertex ===
    providers.insert(
        "vertex/gemini".to_string(),
        ProviderConfig::new(
            "vertex/gemini",
            "GOOGLE_VERTEX_PROJECT",
            "https://{location}-aiplatform.googleapis.com/v1",
            "gemini-1.5-pro",
        ),
    );
    providers.insert(
        "vertex/claude".to_string(),
        ProviderConfig::new(
            "vertex/claude",
            "GOOGLE_VERTEX_PROJECT",
            "https://{location}-aiplatform.googleapis.com/v1",
            "claude-3-5-sonnet-v2",
        ),
    );

    // === Cohere ===
    providers.insert(
        "cohere/command".to_string(),
        ProviderConfig::new(
            "cohere/command",
            "COHERE_API_KEY",
            "https://api.cohere.ai/v1",
            "command-r-plus",
        ),
    );
    providers.insert(
        "cohere/command-light".to_string(),
        ProviderConfig::new(
            "cohere/command-light",
            "COHERE_API_KEY",
            "https://api.cohere.ai/v1",
            "command-r-plus-light",
        ),
    );
    providers.insert(
        "cohere/embed".to_string(),
        ProviderConfig::new(
            "cohere/embed",
            "COHERE_API_KEY",
            "https://api.cohere.ai/v1",
            "embed-english-v3.0",
        ),
    );

    // === Mistral AI ===
    providers.insert(
        "mistral/open".to_string(),
        ProviderConfig::new(
            "mistral/open",
            "MISTRAL_API_KEY",
            "https://api.mistral.ai/v1",
            "mistral-large-latest",
        ),
    );
    providers.insert(
        "mistral/small".to_string(),
        ProviderConfig::new(
            "mistral/small",
            "MISTRAL_API_KEY",
            "https://api.mistral.ai/v1",
            "mistral-small-latest",
        ),
    );
    providers.insert(
        "mistral/codestral".to_string(),
        ProviderConfig::new(
            "mistral/codestral",
            "MISTRAL_API_KEY",
            "https://api.mistral.ai/v1",
            "codestral-latest",
        ),
    );
    providers.insert(
        "mistral/math".to_string(),
        ProviderConfig::new(
            "mistral/math",
            "MISTRAL_API_KEY",
            "https://api.mistral.ai/v1",
            "open-math-large-latest",
        ),
    );
    providers.insert(
        "mistral/codestral-mamba".to_string(),
        ProviderConfig::new(
            "mistral/codestral-mamba",
            "MISTRAL_API_KEY",
            "https://api.mistral.ai/v1",
            "codestral-mamba-latest",
        ),
    );

    // === Together AI ===
    providers.insert(
        "together/llama".to_string(),
        ProviderConfig::new(
            "together/llama",
            "TOGETHER_API_KEY",
            "https://api.together.ai/v1",
            "meta-llama/Llama-3-70b-chat",
        ),
    );
    providers.insert(
        "together/qwen".to_string(),
        ProviderConfig::new(
            "together/qwen",
            "TOGETHER_API_KEY",
            "https://api.together.ai/v1",
            "Qwen/Qwen2-72B-Instruct",
        ),
    );
    providers.insert(
        "together/deepseek".to_string(),
        ProviderConfig::new(
            "together/deepseek",
            "TOGETHER_API_KEY",
            "https://api.togetherai.xyz/v1",
            "deepseek-ai/DeepSeek-V3",
        ),
    );
    providers.insert(
        "together/mixtral".to_string(),
        ProviderConfig::new(
            "together/mixtral",
            "TOGETHER_API_KEY",
            "https://api.together.ai/v1",
            "mistralai/Mixtral-8x22B-Instruct",
        ),
    );
    providers.insert(
        "together/falcon".to_string(),
        ProviderConfig::new(
            "together/falcon",
            "TOGETHER_API_KEY",
            "https://api.together.ai/v1",
            "tiiuae/falcon-180b-chat",
        ),
    );
    providers.insert(
        "together/yi".to_string(),
        ProviderConfig::new(
            "together/yi",
            "TOGETHER_API_KEY",
            "https://api.together.ai/v1",
            "01-ai/Yi-Large",
        ),
    );
    providers.insert(
        "together/mistral".to_string(),
        ProviderConfig::new(
            "together/mistral",
            "TOGETHER_API_KEY",
            "https://api.together.ai/v1",
            "mistralai/Mistral-7B-Instruct",
        ),
    );
    providers.insert(
        "together/codellama".to_string(),
        ProviderConfig::new(
            "together/codellama",
            "TOGETHER_API_KEY",
            "https://api.together.ai/v1",
            "codellama/CodeLlama-70b-Instruct",
        ),
    );

    // === Groq ===
    providers.insert(
        "groq/llama".to_string(),
        ProviderConfig::new(
            "groq/llama",
            "GROQ_API_KEY",
            "https://api.groq.com/openai/v1",
            "llama-3.1-70b-versatile",
        ),
    );
    providers.insert(
        "groq/mixtral".to_string(),
        ProviderConfig::new(
            "groq/mixtral",
            "GROQ_API_KEY",
            "https://api.groq.com/openai/v1",
            "mixtral-8x7b-32768",
        ),
    );
    providers.insert(
        "groq/gemma".to_string(),
        ProviderConfig::new(
            "groq/gemma",
            "GROQ_API_KEY",
            "https://api.groq.com/openai/v1",
            "gemma2-9b-it",
        ),
    );

    // === Perplexity ===
    providers.insert(
        "perplexity/sonar".to_string(),
        ProviderConfig::new(
            "perplexity/sonar",
            "PERPLEXITY_API_KEY",
            "https://api.perplexity.ai",
            "sonar",
        ),
    );
    providers.insert(
        "perplexity/sonar-pro".to_string(),
        ProviderConfig::new(
            "perplexity/sonar-pro",
            "PERPLEXITY_API_KEY",
            "https://api.perplexity.ai",
            "sonar-pro",
        ),
    );

    // === Cerebras ===
    providers.insert(
        "cerebras/llama".to_string(),
        ProviderConfig::new(
            "cerebras/llama",
            "CEREBRAS_API_KEY",
            "https://api.cerebras.ai/v1",
            "llama-3.3-70b",
        ),
    );
    providers.insert(
        "cerebras/qwen".to_string(),
        ProviderConfig::new(
            "cerebras/qwen",
            "CEREBRAS_API_KEY",
            "https://api.cerebras.ai/v1",
            "qwen-2.5-72b",
        ),
    );

    // === Cloudflare Workers AI ===
    providers.insert(
        "cloudflare/llama".to_string(),
        ProviderConfig::new(
            "cloudflare/llama",
            "CLOUDFLARE_API_TOKEN",
            "https://api.cloudflare.com/client/v4/ai",
            "@cf/meta/llama-3.1-70b-instruct",
        ),
    );
    providers.insert(
        "cloudflare/phi".to_string(),
        ProviderConfig::new(
            "cloudflare/phi",
            "CLOUDFLARE_API_TOKEN",
            "https://api.cloudflare.com/client/v4/ai",
            "@cf/microsoft/phi-2",
        ),
    );
    providers.insert(
        "cloudflare/mistral".to_string(),
        ProviderConfig::new(
            "cloudflare/mistral",
            "CLOUDFLARE_API_TOKEN",
            "https://api.cloudflare.com/client/v4/ai",
            "@cf/mistral/mistral-7b-instruct",
        ),
    );
    providers.insert(
        "cloudflare/gemma".to_string(),
        ProviderConfig::new(
            "cloudflare/gemma",
            "CLOUDFLARE_API_TOKEN",
            "https://api.cloudflare.com/client/v4/ai",
            "@cf/google/gemma-2-27b-it",
        ),
    );

    // === SambaNova ===
    providers.insert(
        "sambanova/llama".to_string(),
        ProviderConfig::new(
            "sambanova/llama",
            "SAMBANOVA_API_KEY",
            "https://api.sambanova.ai/api繼1/models",
            "Meta-Llama-3.1-70B-Instruct",
        ),
    );
    providers.insert(
        "sambanova/qwen".to_string(),
        ProviderConfig::new(
            "sambanova/qwen",
            "SAMBANOVA_API_KEY",
            "https://api.sambanova.ai/api/v1/models",
            "Qwen2.5-72B-Instruct",
        ),
    );
    providers.insert(
        "sambanova/mistral".to_string(),
        ProviderConfig::new(
            "sambanova/mistral",
            "SAMBANOVA_API_KEY",
            "https://api.sambanova.ai/api/v1/models",
            "Mistral-7B-Instruct",
        ),
    );

    // === Fireworks AI ===
    providers.insert(
        "fireworks/llama".to_string(),
        ProviderConfig::new(
            "fireworks/llama",
            "FIREWORKS_API_KEY",
            "https://api.fireworks.ai/v1",
            "accounts/fireworks/models/llama-v3-70b-instruct",
        ),
    );
    providers.insert(
        "fireworks/qwen".to_string(),
        ProviderConfig::new(
            "fireworks/qwen",
            "FIREWORKS_API_KEY",
            "https://api.fireworks.ai/v1",
            "accounts/fireworks/models/qwen2-72b-instruct",
        ),
    );
    providers.insert(
        "fireworks/mixtral".to_string(),
        ProviderConfig::new(
            "fireworks/mixtral",
            "FIREWORKS_API_KEY",
            "https://api.fireworks.ai/v1",
            "accounts/fireworks/models/mixtral-8x22b-instruct",
        ),
    );
    providers.insert(
        "fireworks/firefunction".to_string(),
        ProviderConfig::new(
            "fireworks/firefunction",
            "FIREWORKS_API_KEY",
            "https://api.fireworks.ai/v1",
            "accounts/fireworks/models/firefunction-v2",
        ),
    );

    // === Lepton AI ===
    providers.insert(
        "lepton/llama".to_string(),
        ProviderConfig::new(
            "lepton/llama",
            "LEPTON_API_KEY",
            "https://api.lepton.ai/v1",
            "llama-3.1-70b",
        ),
    );
    providers.insert(
        "lepton/qwen".to_string(),
        ProviderConfig::new(
            "lepton/qwen",
            "LEPTON_API_KEY",
            "https://api.lepton.ai/v1",
            "qwen2-72b",
        ),
    );
    providers.insert(
        "lepton/mixtral".to_string(),
        ProviderConfig::new(
            "lepton/mixtral",
            "LEPTON_API_KEY",
            "https://api.lepton.ai/v1",
            "mixtral-8x7b",
        ),
    );

    // === llamasearch ===
    providers.insert(
        "llamasearch/llama".to_string(),
        ProviderConfig::new(
            "llamasearch/llama",
            "LLAMASEARCH_API_KEY",
            "https://api.llamasearch.io/v1",
            "meta-llama/Llama-3.1-70B-Instruct",
        ),
    );
    providers.insert(
        "llamasearch/phi".to_string(),
        ProviderConfig::new(
            "llamasearch/phi",
            "LLAMASEARCH_API_KEY",
            "https://api.llamasearch.io/v1",
            "microsoft/Phi-3-medium-128k-instruct",
        ),
    );

    // === AI21 ===
    providers.insert(
        "ai21/jamba".to_string(),
        ProviderConfig::new(
            "ai21/jamba",
            "AI21_API_KEY",
            "https://api.ai21.com/v1",
            "jamba-1.5-large",
        ),
    );
    providers.insert(
        "ai21/jamba-mini".to_string(),
        ProviderConfig::new(
            "ai21/jamba-mini",
            "AI21_API_KEY",
            "https://api.ai21.com/v1",
            "jamba-1.5-mini",
        ),
    );

    // === Replicate ===
    providers.insert(
        "replicate/llama".to_string(),
        ProviderConfig::new(
            "replicate/llama",
            "REPLICATE_API_TOKEN",
            "https://api.replicate.com/v1",
            "meta/llama-3-70b-instruct",
        ),
    );
    providers.insert(
        "replicate/mistral".to_string(),
        ProviderConfig::new(
            "replicate/mistral",
            "REPLICATE_API_TOKEN",
            "https://api.replicate.com/v1",
            "mistralai/mistral-7b-instruct-v0.3",
        ),
    );
    providers.insert(
        "replicate/qwen".to_string(),
        ProviderConfig::new(
            "replicate/qwen",
            "REPLICATE_API_TOKEN",
            "https://api.replicate.com/v1",
            "qwen/qwen2-72b-instruct",
        ),
    );

    providers
}

pub fn list_providers() -> Vec<ProviderConfig> {
    let providers = PROVIDERS.lock().unwrap();
    let mut list: Vec<_> = providers.values().cloned().collect();
    list.sort_by(|a, b| b.priority.cmp(&a.priority));
    list
}

pub fn get_provider(name: &str) -> Option<ProviderConfig> {
    PROVIDERS.lock().unwrap().get(name).cloned()
}

pub fn get_active_provider() -> Option<ProviderConfig> {
    let providers = PROVIDERS.lock().unwrap();
    let mut candidates: Vec<_> = providers.values().filter(|p| p.enabled).cloned().collect();

    candidates.sort_by(|a, b| b.priority.cmp(&a.priority));

    for p in candidates {
        if std::env::var(&p.api_key_env).is_ok() {
            return Some(p);
        }
    }

    None
}

pub fn set_provider(name: &str, enabled: bool) -> Result<String, String> {
    let mut providers = PROVIDERS.lock().unwrap();

    if let Some(p) = providers.get_mut(name) {
        p.enabled = enabled;
        return Ok(format!(
            "Provider '{}' {}",
            name,
            if enabled { "enabled" } else { "disabled" }
        ));
    }

    Err(format!("Provider '{}' not found", name))
}

pub fn set_priority(name: &str, priority: u32) -> Result<String, String> {
    let mut providers = PROVIDERS.lock().unwrap();

    if let Some(p) = providers.get_mut(name) {
        p.priority = priority;
        return Ok(format!("Set '{}' priority to {}", name, priority));
    }

    Err(format!("Provider '{}' not found", name))
}

pub fn add_custom_provider(config: ProviderConfig) -> Result<String, String> {
    let name = config.name.clone();
    PROVIDERS.lock().unwrap().insert(name.clone(), config);
    Ok(format!("Added provider: {}", name))
}

pub fn remove_provider(name: &str) -> Result<String, String> {
    let mut providers = PROVIDERS.lock().unwrap();

    if providers.remove(name).is_some() {
        return Ok(format!("Removed provider: {}", name));
    }

    Err(format!("Provider '{}' not found", name))
}

pub fn get_api_key(provider_name: &str) -> Option<String> {
    if let Some(p) = get_provider(provider_name) {
        return std::env::var(&p.api_key_env).ok();
    }
    None
}

pub fn list_provider_commands() {
    println!("\n\x1b[36m🔌 Multi-Provider Support\x1b[0m");
    println!("\nUsage:");
    println!("  devutils provider list");
    println!("  devutils provider use <name>");
    println!("  devutils provider set <name> --priority <n>");
    println!("  devutils provider add <name> <api-key> <base-url> <model>");
    println!("  devutils provider remove <name>");
    println!("\nProviders:");
    println!("  openai     - OpenAI GPT-4");
    println!("  anthropic - Anthropic Claude");
    println!("  google    - Google Gemini");
    println!("  deepseek  - DeepSeek");
    println!("  ollama   - Local Ollama (localhost:11434)");
    println!("  xai       - xAI Grok");
    println!("  local     - Local llama.cpp");
    println!("\nEnvironment Variables:");
    println!("  OPENAI_API_KEY, ANTHROPIC_API_KEY, GOOGLE_API_KEY, etc.");
}
