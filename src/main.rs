use bevy::prelude::*;
use bevy::math::bounding::{
  Aabb2d,
  BoundingVolume,
  IntersectsVolume
};

fn main() {

    App::new()
    .add_plugins(DefaultPlugins)
    .insert_resource(Score { player: 0, ai: 0 })
    .add_systems(Startup, (
        spawn_ball,
        spawn_paddles,
        spawn_gutter,
        spawn_scoreboard,
        create_camera))
    .add_systems (FixedUpdate,
      (
        transform_position, 
        move_ball.before(transform_position),
        handle_collisions.after(move_ball),
        move_paddles.before(transform_position),
        handle_player_input.before(move_paddles),
        constrain_paddle_position.after(move_paddles),
        detect_goal.after(move_ball),
        update_scoreboard,
        move_ai,
    ))
    .add_observer(reset_ball)
    .add_observer(update_score)
    .run();
}
// Ball Section ----------->
const BALL_SIZE: f32 = 20.1;
const BALL_SHAPE : Circle = Circle::new(BALL_SIZE);
const BALL_COLOR : Color = Color::srgb(1.0, 0.0, 0.0);

fn spawn_ball (
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
    ) {
    println!("Spawning Empty ball...");

    let mesh = meshes.add(BALL_SHAPE);
    let material = materials.add(BALL_COLOR);

    commands.spawn((Ball, Mesh2d(mesh), MeshMaterial2d(material))); // Create new entity
}

// Paddle Section --------->
const PADDLE_SHAPE : Rectangle = Rectangle::new(20., 50.);
const PADDLE_COLOR : Color = Color::srgb(0.0, 0.0, 1.0);
const PADDLE_COLOR_AI : Color = Color::srgb(0.0, 1.0, 0.0);


// Create Paddle entity
fn spawn_paddles (
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
    ) {
    println!("Spawning Paddles...");

    let mesh = meshes.add(PADDLE_SHAPE);
    let material_player = materials.add(PADDLE_COLOR);
    let material_ai = materials.add(PADDLE_COLOR_AI);
    let half_windows_size = window.resolution.size() / 2.;
    let padding = 20.;
    
    let player_position = Vec2::new(-half_windows_size.x + padding , 0.);
    let ai_position = Vec2::new(half_windows_size.x - padding , 0.);

    commands.spawn((Player, Paddle, Mesh2d(mesh.clone()), MeshMaterial2d(material_player), Position(player_position))); 
    commands.spawn((Ai, Paddle,Mesh2d(mesh.clone()), MeshMaterial2d(material_ai), Position(ai_position))); 
}

fn create_camera (mut commands: Commands) {
    commands.spawn(Camera2d);
}

/*
    Transform : 1. Translation : Position of Ball (x:0 , y:12)
                2. Mesh2d : Shape of Ball ( Circle , Rectangle, ...)
                3. MeshMaterial2d : Structure (Color, Texture, ....)
*/ 

// ---------------- structs ----------------------
const BALL_SPEED: f32 = 2.1;

#[derive(Component, Default)]
#[require(Transform)] 
struct Position(Vec2);

#[derive(Component)]
#[require(
    Position,
    Velocity = Velocity(Vec2::new(BALL_SPEED, BALL_SPEED)),
    Collider = Collider(Rectangle::new(BALL_SIZE, BALL_SIZE))
    )]
struct Ball;

#[derive(Component, Default)]
#[require(
    Position,
    Velocity,
    Collider = Collider(PADDLE_SHAPE)
)]
struct Paddle;

#[derive(Component, Default)]
struct Velocity(Vec2);

// برخورد دهنده
// باکس دور اشیاء میکشه که برخورد هارو شناسایی کنیم
#[derive(Component, Default)]
struct Collider(Rectangle);

impl Collider {
  fn half_size(&self) -> Vec2 {
    self.0.half_size
  }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
  Left,
  Right,
  Top,
  Bottom,
}
// -------------------------------------------

fn transform_position(mut positionables: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in &mut positionables {
        transform.translation = position.0.extend(0.);
    }
}

fn move_ball (ball : Single<(&mut Position, &Velocity), With<Ball>>) {
    let (mut position, velocity) = ball.into_inner();
    position.0 += velocity.0 *  BALL_SPEED;      
}

// Returns `Some` if `ball` collides with `wall`. The returned `Collision` is the
// side of `wall` that `ball` hit.
fn collide_with_side (ball:Aabb2d ,wall: Aabb2d) -> Option<Collision> {

    if !ball.intersects(&wall) {
        return None;
    }

    // مختصات نزدیکترین فاصله از دیوار به توپ
    let closed_to_ball = wall.closest_point(ball.center());

    // فاصله بین مرکز توپ و نزدیکترین فاصله از دیوار به توپ
    let offset = ball.center() - closed_to_ball;

    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else{
        Collision::Bottom
    };

    Some(side)
}

fn handle_collisions(
  ball: Single<(&mut Velocity, &Position, &Collider), With<Ball>>,
  other_things: Query<(&Position, &Collider), Without<Ball>>,
) {
  let (mut ball_velocity, ball_position, ball_collider) = ball.into_inner();

  for (other_position, other_collider) in &other_things {
    if let Some(collision) = collide_with_side(
      Aabb2d::new(ball_position.0, ball_collider.half_size()),
      Aabb2d::new(other_position.0, other_collider.half_size()),
    ) {
      match collision {
        Collision::Left => {
          ball_velocity.0.x *= -1.;
        }
        Collision::Right => {
          ball_velocity.0.x *= -1.;
        }
        Collision::Top => {
          ball_velocity.0.y *= -1.;
        }
        Collision::Bottom => {
          ball_velocity.0.y *= -1.;
        }
      }
    }
  }
}

// ---------- player-ai --------------

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Ai;

// -------------- gutters ---------------

#[derive(Component)]
#[require(Position, Collider)]
struct Gutter;

const GUTTER_COLOR: Color = Color::srgb(0., 10., 10.);
const GUTTER_HEIGHT: f32 = 20.;

fn spawn_gutter (
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  windows: Single<&Window>
){  
    
  let gutter_rectangle = Rectangle::new(windows.resolution.width(), GUTTER_HEIGHT);
  let mesh = meshes.add(gutter_rectangle);
  let material = materials.add(GUTTER_COLOR);

  let padding = 40.;

  let gutter_position_top = Vec2::new(0., windows.resolution.height() / 2. - padding);

  commands.spawn(
    (
    Gutter,
    Mesh2d(mesh.clone()),
    MeshMaterial2d(material.clone()),
    Position(gutter_position_top),
    Collider(gutter_rectangle),
  ));

  let gutter_position_bottom = Vec2::new(0., -windows.resolution.height() / 2. + padding);

  commands.spawn(
    (
    Gutter,
    Mesh2d(mesh),
    MeshMaterial2d(material),
    Position(gutter_position_bottom),
    Collider(gutter_rectangle),
  ));
}

// -------------- Moving Paddles -------------

const PADDLE_SPEED: f32 = 5.;

fn handle_player_input(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut paddle_velocity: Single<&mut Velocity, With<Player>>,
) {
  if keyboard_input.pressed(KeyCode::ArrowUp) {
    paddle_velocity.0.y = PADDLE_SPEED;
  } else if keyboard_input.pressed(KeyCode::ArrowDown) {
    paddle_velocity.0.y = -PADDLE_SPEED;
  } else {
    paddle_velocity.0.y = 0.;
  }
}

fn move_paddles(mut paddles: Query<(&mut Position, &Velocity), With<Paddle>>) {
  for (mut position, velocity) in &mut paddles {
    position.0 += velocity.0;
  }
}

// ---------------- محدود کردن حرکت پدل ها -----------------

fn constrain_paddle_position(
  mut paddles: Query<
    (&mut Position, &Collider),
    (With<Paddle>, Without<Gutter>),
  >,
  gutters: Query<(&Position, &Collider), (With<Gutter>, Without<Paddle>)>,
) {
  for (mut paddle_position, paddle_collider) in &mut paddles {
    for (gutter_position, gutter_collider) in &gutters {
      let paddle_aabb =
        Aabb2d::new(paddle_position.0, paddle_collider.half_size());
      let gutter_aabb =
        Aabb2d::new(gutter_position.0, gutter_collider.half_size());

      if let Some(collision) = collide_with_side(paddle_aabb, gutter_aabb) {
        match collision {
          Collision::Top => {
            paddle_position.0.y = gutter_position.0.y
              + gutter_collider.half_size().y
              + paddle_collider.half_size().y;
          }
          Collision::Bottom => {
            paddle_position.0.y = gutter_position.0.y
              - gutter_collider.half_size().y
              - paddle_collider.half_size().y;
          }
          _ => {}
        }
      }
    }
  }
}

// --------------- Score Section ----------------

#[derive(Resource)]
struct Score {
  player: u32,
  ai: u32,
}

#[derive(EntityEvent)]
struct Scored {
  #[event_target]
  scorer: Entity,
}

fn detect_goal(
  ball: Single<(&Position, &Collider), With<Ball>>,
  player: Single<Entity, (With<Player>, Without<Ai>)>,
  ai: Single<Entity, (With<Ai>, Without<Player>)>,
  window: Single<&Window>,
  mut commands: Commands,
) {
  let (ball_position, ball_collider) = ball.into_inner();
  let half_window_size = window.resolution.size() / 2.;

  if ball_position.0.x - ball_collider.half_size().x > half_window_size.x {
    commands.trigger(Scored { scorer: *player });
  }

  if ball_position.0.x + ball_collider.half_size().x < -half_window_size.x {
    commands.trigger(Scored { scorer: *ai });
  }
}

fn reset_ball(
  _event: On<Scored>,
  ball: Single<(&mut Position, &mut Velocity), With<Ball>>,
) {
  let (mut ball_position, mut ball_velocity) = ball.into_inner();
  ball_position.0 = Vec2::ZERO;
  ball_velocity.0 = Vec2::new(BALL_SPEED, BALL_SPEED);
}

fn update_score(
  event: On<Scored>,
  mut score: ResMut<Score>,
  is_ai: Query<&Ai>,
  is_player: Query<&Player>,
) {
  if is_ai.get(event.scorer).is_ok() {
    score.ai += 1;
    info!("AI scored! {} - {}", score.player, score.ai);
  }

  if is_player.get(event.scorer).is_ok() {
    score.player += 1;
    info!("Player scored! {} - {}", score.player, score.ai);
  }
}
// --------------------- Display Score ----------------

#[derive(Component)]
struct PlayerScore;

#[derive(Component)]
struct AiScore;


fn spawn_scoreboard(mut commands: Commands) {
  // Create a container that will center everything
  let container = Node {
    width: percent(100.0),
    height: percent(100.0),
    justify_content: JustifyContent::Center,
    ..default()
  };

  // Then add a container for the text
  let header = Node {
    width: px(200.),
    height: px(100.),
    ..default()
  };

  // The players score on the left hand side
  let player_score = (
    PlayerScore,
    Text::new("0"),
    TextFont::from_font_size(72.0),
    TextColor(Color::WHITE),
    TextLayout::new_with_justify(Justify::Center),
    Node {
      position_type: PositionType::Absolute,
      top: px(5.0),
      left: px(25.0),
      ..default()
    },
  );

  // The AI score on the right hand side
  let ai_score = (
    AiScore,
    Text::new("0"),
    TextFont::from_font_size(72.0),
    TextColor(Color::WHITE),
    TextLayout::new_with_justify(Justify::Center),
    Node {
      position_type: PositionType::Absolute,
      top: px(5.0),
      right: px(25.0),
      ..default()
    },
  );

  commands.spawn((
    container,
    children![(header, children![player_score, ai_score])],
  ));
}

fn update_scoreboard(
  mut player_score: Single<&mut Text, (With<PlayerScore>, Without<AiScore>)>,
  mut ai_score: Single<&mut Text, (With<AiScore>, Without<PlayerScore>)>,
  score: Res<Score>,
) {
  if score.is_changed() {
    player_score.0 = score.player.to_string();
    ai_score.0 = score.ai.to_string();
  }
}

// ----------------- ADD Ai to this --------------

fn move_ai(
  ai: Single<(&mut Velocity, &Position), With<Ai>>,
  ball: Single<&Position, With<Ball>>,
) {
  let (mut velocity, position) = ai.into_inner();
  let a_to_b = ball.0 - position.0;
  velocity.0.y = a_to_b.y.signum() * PADDLE_SPEED;
}