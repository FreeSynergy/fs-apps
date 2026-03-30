use fs_bots::{
    bot_strategy::BotAction,
    model::{ApprovalAction, BotKind, ChannelTarget, MessagingBot, Platform},
};

fn make_bot(kind: BotKind) -> MessagingBot {
    MessagingBot {
        id: "test".into(),
        name: "Test Bot".into(),
        kind,
        enabled: true,
        targets: vec![],
        recent_broadcasts: vec![],
        pending_approvals: vec![],
    }
}

fn enabled_target() -> ChannelTarget {
    ChannelTarget {
        platform: "telegram".into(),
        name: "@test".into(),
        id: "t1".into(),
        enabled: true,
    }
}

#[test]
fn bot_kind_label_broadcast() {
    assert_eq!(BotKind::Broadcast.label(), "Broadcast");
}

#[test]
fn bot_kind_label_gatekeeper() {
    assert_eq!(BotKind::Gatekeeper.label(), "Gatekeeper");
}

#[test]
fn bot_kind_label_monitor() {
    assert_eq!(BotKind::Monitor.label(), "Monitor");
}

#[test]
fn platform_all_has_seven_entries() {
    assert_eq!(Platform::all().len(), 7);
}

#[test]
fn platform_telegram_label() {
    assert_eq!(Platform::Telegram.label(), "Telegram");
}

#[test]
fn platform_matrix_label() {
    assert_eq!(Platform::Matrix.label(), "Matrix");
}

#[test]
fn platform_credential_fields_telegram_nonempty() {
    assert!(!Platform::Telegram.credential_fields().is_empty());
}

#[test]
fn broadcast_strategy_rejects_resolve_approval() {
    let mut bot = make_bot(BotKind::Broadcast);
    let strategy = BotKind::Broadcast.strategy();
    let result = strategy.apply(
        &mut bot,
        BotAction::ResolveApproval {
            id: "x".into(),
            action: ApprovalAction::Allow,
        },
    );
    assert!(result.is_err());
}

#[test]
fn gatekeeper_strategy_rejects_send_broadcast() {
    let mut bot = make_bot(BotKind::Gatekeeper);
    bot.targets.push(enabled_target());
    let strategy = BotKind::Gatekeeper.strategy();
    let result = strategy.apply(
        &mut bot,
        BotAction::SendBroadcast {
            message: "hello".into(),
            target_count: 1,
        },
    );
    assert!(result.is_err());
}
