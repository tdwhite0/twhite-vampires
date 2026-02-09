use bevy::prelude::*;
use crate::components::*;

// === Notification Toasts ===
const TOAST_LIFETIME: f32 = 2.5;
const TOAST_SLIDE_DURATION: f32 = 0.3;
const TOAST_FADE_START: f32 = 1.8;
const TOAST_SLOT_HEIGHT: f32 = 36.0;
const TOAST_BASE_BOTTOM: f32 = 80.0;

pub fn spawn_notification_toasts(
    mut commands: Commands,
    mut events: MessageReader<NotificationEvent>,
    mut existing: Query<&mut NotificationToast>,
) {
    for event in events.read() {
        // Bump existing toasts up one slot
        for mut toast in existing.iter_mut() {
            toast.slot += 1;
        }

        // Spawn new toast at slot 0
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(TOAST_BASE_BOTTOM),
                    left: Val::Percent(0.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                NotificationToast {
                    age: 0.0,
                    lifetime: TOAST_LIFETIME,
                    slot: 0,
                },
                // Don't tag with GameEntity — toasts should persist across brief state changes
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(&event.message),
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(event.kind.color()),
                ));
            });
    }
}

pub fn update_notification_toasts(
    mut commands: Commands,
    mut query: Query<(Entity, &mut NotificationToast, &mut Node, &Children)>,
    mut text_colors: Query<&mut TextColor>,
    time: Res<Time>,
) {
    for (entity, mut toast, mut node, children) in query.iter_mut() {
        toast.age += time.delta_secs();

        if toast.age >= toast.lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        // Calculate target bottom position based on slot
        let target_bottom = TOAST_BASE_BOTTOM + toast.slot as f32 * TOAST_SLOT_HEIGHT;

        // Slide-up animation during first TOAST_SLIDE_DURATION seconds
        let slide_progress = (toast.age / TOAST_SLIDE_DURATION).min(1.0);
        // Ease-out: 1 - (1-t)^2
        let eased = 1.0 - (1.0 - slide_progress) * (1.0 - slide_progress);
        // Start 30px below target, animate to target
        let current_bottom = target_bottom - 30.0 * (1.0 - eased);
        node.bottom = Val::Px(current_bottom);

        // Fade out during last phase
        let alpha = if toast.age > TOAST_FADE_START {
            let fade_progress = (toast.age - TOAST_FADE_START) / (toast.lifetime - TOAST_FADE_START);
            1.0 - fade_progress
        } else if toast.age < TOAST_SLIDE_DURATION {
            eased
        } else {
            1.0
        };

        // Apply alpha to child text
        for child in children.iter() {
            if let Ok(mut color) = text_colors.get_mut(child) {
                let base = color.0.to_srgba();
                *color = TextColor(Color::srgba(base.red, base.green, base.blue, alpha));
            }
        }
    }
}
