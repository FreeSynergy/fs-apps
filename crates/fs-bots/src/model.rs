use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── TomlConfig ─────────────────────────────────────────────────────────────────

/// Shared persistence behaviour for TOML-backed config files.
/// Implementors only provide `config_path()`; `load_config()` and `save()` are free.
pub trait TomlConfig: Default + Serialize + for<'de> Deserialize<'de> + Sized {
    fn config_path() -> PathBuf;

    fn load_config() -> Self {
        let content = std::fs::read_to_string(Self::config_path()).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    }

    fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }
}

// ── BotKind ───────────────────────────────────────────────────────────────────

/// All display properties for a `BotKind` variant — single source of truth.
pub struct BotKindMeta {
    pub label: &'static str,
    pub icon:  &'static str,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BotKind {
    Broadcast,
    Gatekeeper,
    Monitor,
    Digest,
    UserBot,
}

impl BotKind {
    /// Single match block — all display properties in one place.
    pub fn meta(&self) -> BotKindMeta {
        match self {
            Self::Broadcast  => BotKindMeta { label: "Broadcast",          icon: "📢" },
            Self::Gatekeeper => BotKindMeta { label: "Gatekeeper",         icon: "🔒" },
            Self::Monitor    => BotKindMeta { label: "Monitor",            icon: "📊" },
            Self::Digest     => BotKindMeta { label: "Digest",             icon: "📋" },
            Self::UserBot    => BotKindMeta { label: "Personal Assistant", icon: "🤖" },
        }
    }

    pub fn label(&self) -> &'static str { self.meta().label }
    pub fn icon(&self)  -> &'static str { self.meta().icon  }
}

// ── Platform ──────────────────────────────────────────────────────────────────

/// A credential field definition: (name, placeholder, is_secret).
pub struct CredentialField {
    pub name:        &'static str,
    pub placeholder: &'static str,
    pub is_secret:   bool,
}

/// All display properties for a `Platform` variant — single source of truth.
pub struct PlatformMeta {
    pub label: &'static str,
    pub icon:  &'static str,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Telegram,
    Matrix,
    Discord,
    RocketChat,
    Mattermost,
    Slack,
    XMPP,
}

impl Platform {
    pub fn all() -> &'static [Platform] {
        &[
            Platform::Telegram,
            Platform::Matrix,
            Platform::Discord,
            Platform::RocketChat,
            Platform::Mattermost,
            Platform::Slack,
            Platform::XMPP,
        ]
    }

    /// Single match block — all display properties in one place.
    pub fn meta(&self) -> PlatformMeta {
        match self {
            Self::Telegram   => PlatformMeta { label: "Telegram",    icon: "✈"  },
            Self::Matrix     => PlatformMeta { label: "Matrix",      icon: "🔷" },
            Self::Discord    => PlatformMeta { label: "Discord",     icon: "🎮" },
            Self::RocketChat => PlatformMeta { label: "Rocket.Chat", icon: "🚀" },
            Self::Mattermost => PlatformMeta { label: "Mattermost",  icon: "📡" },
            Self::Slack      => PlatformMeta { label: "Slack",       icon: "💼" },
            Self::XMPP       => PlatformMeta { label: "XMPP",        icon: "💬" },
        }
    }

    pub fn label(&self) -> &'static str { self.meta().label }
    pub fn icon(&self)  -> &'static str { self.meta().icon  }

    pub fn credential_fields(&self) -> Vec<CredentialField> {
        match self {
            Self::Telegram => vec![
                CredentialField { name: "Bot Token",       placeholder: "1234567890:ABCdef...", is_secret: true  },
                CredentialField { name: "Phone (UserBot)", placeholder: "+49123456789",         is_secret: false },
            ],
            Self::Matrix => vec![
                CredentialField { name: "Homeserver",   placeholder: "https://matrix.example.com", is_secret: false },
                CredentialField { name: "Access Token", placeholder: "syt_...",                    is_secret: true  },
            ],
            Self::Discord => vec![
                CredentialField { name: "Bot Token", placeholder: "MTx....", is_secret: true },
            ],
            Self::RocketChat => vec![
                CredentialField { name: "Server URL", placeholder: "https://chat.example.com", is_secret: false },
                CredentialField { name: "Username",   placeholder: "admin_bot",                is_secret: false },
                CredentialField { name: "Password",   placeholder: "...",                      is_secret: true  },
            ],
            Self::Mattermost => vec![
                CredentialField { name: "Server URL",  placeholder: "https://mattermost.example.com", is_secret: false },
                CredentialField { name: "Bot Token",   placeholder: "token...",                       is_secret: true  },
            ],
            Self::Slack => vec![
                CredentialField { name: "Bot Token",     placeholder: "xoxb-...", is_secret: true  },
                CredentialField { name: "App Token",     placeholder: "xapp-...", is_secret: true  },
            ],
            Self::XMPP => vec![
                CredentialField { name: "JID",      placeholder: "bot@example.com", is_secret: false },
                CredentialField { name: "Password", placeholder: "...",             is_secret: true  },
            ],
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Matrix"     => Self::Matrix,
            "Discord"    => Self::Discord,
            "RocketChat" | "Rocket.Chat" => Self::RocketChat,
            "Mattermost" => Self::Mattermost,
            "Slack"      => Self::Slack,
            "XMPP"       => Self::XMPP,
            _            => Self::Telegram,
        }
    }
}

// ── ControlBotAccount ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlBotAccount {
    pub id:          String,
    pub platform:    Platform,
    pub label:       String,
    pub credentials: Vec<(String, String)>, // (field_name, value)
    pub connected:   bool,
}

impl ControlBotAccount {
    /// Factory: validates input and builds an account. Returns `None` if label is empty.
    pub fn create(
        platform:    Platform,
        label:       String,
        credentials: Vec<(String, String)>,
        count:       usize,
    ) -> Option<Self> {
        if label.trim().is_empty() { return None; }
        Some(Self {
            id:        format!("acc-{}", count + 1),
            platform,
            label,
            credentials,
            connected: false,
        })
    }
}

// ── ControlBotConfig ──────────────────────────────────────────────────────────

#[derive(Default, Serialize, Deserialize)]
pub struct ControlBotConfig {
    #[serde(default)]
    pub accounts: Vec<ControlBotAccount>,
}

impl TomlConfig for ControlBotConfig {
    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".config").join("fsn").join("control_bot.toml")
    }
}

impl ControlBotConfig {
    pub fn load() -> Vec<ControlBotAccount> {
        Self::load_config().accounts
    }
}

// ── MessagingBot ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChannelTarget {
    pub platform: String,
    pub name:     String,
    pub id:       String,
    pub enabled:  bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BroadcastRecord {
    pub message:      String,
    pub sent_at:      DateTime<Utc>,
    pub target_count: usize,
}

impl BroadcastRecord {
    pub fn time_ago(&self) -> String {
        let secs = (Utc::now() - self.sent_at).num_seconds();
        if secs < 60        { format!("{secs}s ago") }
        else if secs < 3600 { format!("{}m ago", secs / 60) }
        else                { format!("{}h ago", secs / 3600) }
    }

    pub fn preview(&self, max_len: usize) -> String {
        if self.message.len() > max_len {
            format!("{}…", &self.message[..max_len - 1])
        } else {
            self.message.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendingApproval {
    pub id:            String,
    pub username:      String,
    pub platform:      String,
    pub waiting_since: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MessagingBot {
    pub id:      String,
    pub name:    String,
    pub kind:    BotKind,
    pub enabled: bool,
    pub targets: Vec<ChannelTarget>,
    #[serde(default)]
    pub recent_broadcasts: Vec<BroadcastRecord>,
    #[serde(skip)]
    pub pending_approvals: Vec<PendingApproval>,
}

/// Whether a pending approval was granted or denied.
#[derive(Clone, Debug, PartialEq)]
pub enum ApprovalAction {
    Allow,
    Deny,
}

impl MessagingBot {
    /// Record a sent broadcast message. Returns `false` if the message is empty.
    pub fn send_broadcast(&mut self, message: &str, target_count: usize) -> bool {
        if message.trim().is_empty() { return false; }
        let record = BroadcastRecord {
            message:      message.to_string(),
            sent_at:      Utc::now(),
            target_count,
        };
        self.recent_broadcasts.insert(0, record);
        if self.recent_broadcasts.len() > 20 {
            self.recent_broadcasts.truncate(20);
        }
        true
    }

    /// Remove a pending approval entry (allow or deny — both resolve the request).
    pub fn resolve_approval(&mut self, id: &str, _action: ApprovalAction) {
        self.pending_approvals.retain(|a| a.id != id);
    }

    pub fn demo_bots() -> Vec<Self> {
        let mut broadcast = MessagingBotBuilder::new("broadcast", "Broadcast Bot", BotKind::Broadcast)
            .enabled()
            .target("Telegram", "FreeSynergy Community", "-100123")
            .target("Matrix",   "#general:example.com",  "!abc:example.com")
            .build();
        broadcast.recent_broadcasts = vec![
            BroadcastRecord {
                message:      "New update available…".into(),
                sent_at:      Utc::now() - chrono::Duration::hours(2),
                target_count: 4,
            },
        ];

        let mut gatekeeper = MessagingBotBuilder::new("gatekeeper", "Gatekeeper Bot", BotKind::Gatekeeper)
            .enabled()
            .target("Telegram", "FreeSynergy Community", "-100123")
            .build();
        gatekeeper.pending_approvals = vec![
            PendingApproval {
                id: "1".into(), username: "@alice_t".into(),
                platform: "Telegram".into(),
                waiting_since: Utc::now() - chrono::Duration::minutes(2),
            },
            PendingApproval {
                id: "2".into(), username: "@bob_99".into(),
                platform: "Telegram".into(),
                waiting_since: Utc::now() - chrono::Duration::minutes(4),
            },
        ];

        vec![broadcast, gatekeeper]
    }
}

// ── MessagingBotBuilder ───────────────────────────────────────────────────────

/// Builder for `MessagingBot` — fluent API for construction and test fixtures.
pub struct MessagingBotBuilder {
    id:      String,
    name:    String,
    kind:    BotKind,
    enabled: bool,
    targets: Vec<ChannelTarget>,
}

impl MessagingBotBuilder {
    pub fn new(id: impl Into<String>, name: impl Into<String>, kind: BotKind) -> Self {
        Self { id: id.into(), name: name.into(), kind, enabled: false, targets: vec![] }
    }

    pub fn enabled(mut self) -> Self { self.enabled = true; self }

    pub fn target(
        mut self,
        platform: impl Into<String>,
        name:     impl Into<String>,
        id:       impl Into<String>,
    ) -> Self {
        self.targets.push(ChannelTarget {
            platform: platform.into(),
            name:     name.into(),
            id:       id.into(),
            enabled:  true,
        });
        self
    }

    pub fn build(self) -> MessagingBot {
        MessagingBot {
            id:                self.id,
            name:              self.name,
            kind:              self.kind,
            enabled:           self.enabled,
            targets:           self.targets,
            recent_broadcasts: vec![],
            pending_approvals: vec![],
        }
    }
}

// ── MessagingBotsConfig ───────────────────────────────────────────────────────

#[derive(Default, Serialize, Deserialize)]
pub struct MessagingBotsConfig {
    #[serde(default)]
    pub bots: Vec<MessagingBot>,
}

impl TomlConfig for MessagingBotsConfig {
    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".config").join("fsn").join("messaging_bots.toml")
    }
}

impl MessagingBotsConfig {
    pub fn load() -> Vec<MessagingBot> {
        let cfg = Self::load_config();
        if cfg.bots.is_empty() { MessagingBot::demo_bots() } else { cfg.bots }
    }
}

// ── Groups ────────────────────────────────────────────────────────────────────

/// A messenger room cached locally (mirrors bot-runtime `known_rooms`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CachedRoom {
    pub platform:     String,
    pub room_id:      String,
    pub room_name:    String,
    pub member_count: Option<u32>,
}

/// A manual collection of rooms.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoomCollection {
    pub id:          u32,
    pub name:        String,
    pub description: String,
    /// (platform, room_id) pairs.
    pub members:     Vec<(String, String)>,
}

/// Serialization root for groups.toml.
#[derive(Default, Serialize, Deserialize)]
pub struct GroupsConfig {
    #[serde(default)]
    pub collections:  Vec<RoomCollection>,
    #[serde(default)]
    pub cached_rooms: Vec<CachedRoom>,
}

impl TomlConfig for GroupsConfig {
    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home).join(".config").join("fsn").join("groups.toml")
    }
}

impl GroupsConfig {
    pub fn load() -> Self { Self::load_config() }
}
