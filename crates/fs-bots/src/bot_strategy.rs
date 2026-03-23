/// Strategy pattern for bot-kind-specific validation and action execution.
/// Each `BotKind` carries its own strategy; views call `strategy.apply()` instead
/// of embedding logic directly in closures.
use crate::model::{ApprovalAction, BotKind, MessagingBot};

// ── BotAction ──────────────────────────────────────────────────────────────────

/// All actions that can be applied to a `MessagingBot`.
#[derive(Clone, Debug)]
pub enum BotAction {
    SendBroadcast { message: String, target_count: usize },
    ResolveApproval { id: String, action: ApprovalAction },
}

// ── BotStrategy ────────────────────────────────────────────────────────────────

/// Per-bot-kind strategy: validates configuration and applies actions.
pub trait BotStrategy {
    /// Check that the bot is properly configured for this strategy.
    fn validate(&self, bot: &MessagingBot) -> Result<(), String>;

    /// Apply a `BotAction` to the bot, returning an error on misuse or invalid state.
    fn apply(&self, bot: &mut MessagingBot, action: BotAction) -> Result<(), String>;
}

// ── BroadcastStrategy ─────────────────────────────────────────────────────────

pub struct BroadcastStrategy;

impl BotStrategy for BroadcastStrategy {
    fn validate(&self, bot: &MessagingBot) -> Result<(), String> {
        if bot.targets.is_empty() {
            return Err("No targets configured".into());
        }
        if !bot.targets.iter().any(|t| t.enabled) {
            return Err("No enabled targets".into());
        }
        Ok(())
    }

    fn apply(&self, bot: &mut MessagingBot, action: BotAction) -> Result<(), String> {
        match action {
            BotAction::SendBroadcast { message, target_count } => {
                self.validate(bot)?;
                if !bot.send_broadcast(&message, target_count) {
                    return Err("Message is empty".into());
                }
                Ok(())
            }
            _ => Err("Action not supported by BroadcastStrategy".into()),
        }
    }
}

// ── GatekeeperStrategy ────────────────────────────────────────────────────────

pub struct GatekeeperStrategy;

impl BotStrategy for GatekeeperStrategy {
    fn validate(&self, bot: &MessagingBot) -> Result<(), String> {
        if bot.targets.is_empty() {
            return Err("No targets configured".into());
        }
        Ok(())
    }

    fn apply(&self, bot: &mut MessagingBot, action: BotAction) -> Result<(), String> {
        match action {
            BotAction::ResolveApproval { id, action: approval_action } => {
                self.validate(bot)?;
                bot.resolve_approval(&id, approval_action);
                Ok(())
            }
            _ => Err("Action not supported by GatekeeperStrategy".into()),
        }
    }
}

// ── DefaultStrategy ───────────────────────────────────────────────────────────

pub struct DefaultStrategy;

impl BotStrategy for DefaultStrategy {
    fn validate(&self, _bot: &MessagingBot) -> Result<(), String> { Ok(()) }
    fn apply(&self, _bot: &mut MessagingBot, _action: BotAction) -> Result<(), String> {
        Err("No strategy implemented for this bot type yet".into())
    }
}

// ── BotKind extension ──────────────────────────────────────────────────────────

impl BotKind {
    /// Return the strategy for this bot kind — replaces match blocks in views.
    pub fn strategy(&self) -> Box<dyn BotStrategy> {
        match self {
            Self::Broadcast  => Box::new(BroadcastStrategy),
            Self::Gatekeeper => Box::new(GatekeeperStrategy),
            _                => Box::new(DefaultStrategy),
        }
    }
}
