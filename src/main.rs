use bracket_lib::prelude::*;
use rand::rngs::ThreadRng;  // 新增导入
use rand::Rng;

// -------------------------- 数据结构定义 --------------------------
//玩家属性
struct Player {
    position_x: i32,               // 坐标位置x
    position_y: i32,               // 坐标位置y
    health: u32,                   // 当前血量（初始20）
    max_health: u32,               // 最大血量
    attack: u32,                   // 攻击力（初始2）
    attack_speed: f32,             // 子弹数量（初始2）
    current_weapon: Weapon,        // 当前装备武器
    experience: u32,               // 当前经验值
    level: u32,                    // 当前等级（初始1）
    coins: u32,                    // 金币数量
}

impl Player{
    fn new(current_weapon: Weapon)-> Self{
        let (attack,attack_speed) = match current_weapon {
            Weapon::MachineGun => (2,1.2),
            Weapon::Laser => (4,0.8),
            Weapon::ShotGun => (4,1.0),
        };

        Player{
            position_x: 40,
            position_y: 40,
            health: 20,
            max_health: 20,
            attack,
            attack_speed,
            current_weapon,
            experience: 0,
            level: 1,
            coins: 0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm){
        ctx.set(self.position_x, self.position_y, YELLOW, WHITE, to_cp437('@'));
    }

    fn attack(&mut self) -> Bullet{
        Bullet::new(self)
    }
    fn move_player(&mut self, param: char){
        match param {
            'W' => {
                if self.position_y > 6 {
                    self.position_y -= 1;
                }
            }
            'S' => {
                if self.position_y < SCREEN_HEIGHT - 1 {
                    self.position_y += 1;
                }
            }
            'A' => {
                if self.position_x > 0 {
                    self.position_x -= 1;
                }
            }
            'D' => {
                if self.position_x < SCREEN_WIDTH - 1 {
                    self.position_x += 1;
                }
            }
            _ => {}  // 忽略其他字符
        }
    }

    fn hurt(&mut self, damage: u32){
        if self.health > damage {
            self.health -= damage;
        } 
        else {
            self.health = 0;
        }
    }


}

// 武器类型
enum Weapon {
    MachineGun,
    Laser,
    ShotGun,
}
impl Clone for Weapon {
    fn clone(&self) -> Self {
        match self {
            Weapon::MachineGun => Weapon::MachineGun,
            Weapon::Laser => Weapon::Laser,
            Weapon::ShotGun => Weapon::ShotGun,
        }
    }
}

// 敌人类型
enum EnemyType {
    Scout,       // 侦察机
    HeavyShip,   // 重甲舰
    Carrier,     // 航母（精英）
    Ghost,       // 幽灵战机（精英）
    Destroyer    // BOSS灭星者
}
// 敌人属性
struct Enemy {
    enemy_type: EnemyType,       // 敌人类型（设计文档3.41）
    health: u32,                 // 血量
    max_health: u32,             // 最大血量
    bullet_damage: u32,          // 弹幕伤害
    collision_damage: u32,       // 碰撞伤害
    position_x: i32,             // 当前位置x
    position_y: i32,             // 当前位置y
    is_elites: bool,             // 是否精英怪
    is_boss: bool,               // 是否BOSS
    special_ability: Ability, // 特殊能力（护盾场/死亡反击等）
    direction_x: i32,            // x 方向移动速度
    direction_y: i32,            // y 方向移动速度
    bullets: Vec<Bullet>, // 修改为使用 Bullet 结构体
}
// 敌人技能
enum Ability {
    ShieldField,  // 护盾场血量（50点，设计文档3.42）
    DeathCounter,      // 死亡反击（幽灵战机专属）
    PhaseShift,        // 阶段变换（BOSS专属）
    Nope,              // 小怪无
}

impl Enemy{
    fn new(enemy_type: EnemyType) -> Self {
        let (max_health, health, bullet_damage, collision_damage, is_elites, is_boss, special_ability) = match enemy_type {
            EnemyType::Scout => (10, 10, 0, 1, false, false,Ability::Nope),
            EnemyType::HeavyShip => (50, 50, 3, 3, false, false,Ability::Nope),
            EnemyType::Carrier => (100, 100, 0, 5, true, false,Ability::ShieldField),
            EnemyType::Ghost => (80, 80, 4, 4, true, false,Ability::DeathCounter),
            EnemyType::Destroyer => (200, 200, 8, 8, false, true,Ability::PhaseShift),
        };

        let (direction_x, direction_y) = match enemy_type {
            EnemyType::Scout => (4, 2),
            EnemyType::HeavyShip => (0, 1),
            EnemyType::Carrier => (0, 0),
            EnemyType::Ghost => (3, 3),
            EnemyType::Destroyer => (1, 1),
        };

        Enemy {
            enemy_type,
            health,
            max_health,
            bullet_damage,
            collision_damage,
            position_x: 40,
            position_y: 6,
            is_elites,
            is_boss,
            special_ability,
            direction_x,
            direction_y,
            bullets: Vec::new(), // 初始化子弹列表
        }
    }

    fn render(&mut self, ctx: &mut BTerm){
        ctx.set(self.position_x,self.position_y, WHITE, BLACK,to_cp437('!'));
        
        // 血条渲染（位于敌人上方1格，长度10，红色背景白色边框）
        let blood_bar_length = (self.health as f32 / self.max_health as f32 * 10.0) as i32;
        ctx.set(self.position_x - 5, self.position_y - 1, BLACK, RED, to_cp437('[')); // 左边界
        for i in 0..blood_bar_length {
            ctx.set(self.position_x - 4 + i, self.position_y - 1, RED, BLACK, to_cp437('█')); // 填充部分
        }
        ctx.set(self.position_x - 5 + blood_bar_length, self.position_y - 1, BLACK, RED, to_cp437(']')); // 右边界
    }

    fn move_enemy(&mut self){
        let new_x = self.position_x + self.direction_x;
        let new_y = self.position_y + self.direction_y;

        // 检查 x 边界
        if new_x < 0 || new_x > 79 {
            self.direction_x = 0 - self.direction_x;
        }

        // 检查 y 边界
        if new_y < 6 || new_y > 50 {
            self.direction_y = 0 - self.direction_y;
        }

        self.position_x += self.direction_x;
        self.position_y += self.direction_y;
    }

    // 修改为类似 player 的 attack 方法
    fn attack(&mut self) -> Bullet {
        Bullet::new_enemy(self)
    }

    // 发射子弹
    fn shoot(&mut self) {
        if let EnemyType::HeavyShip = self.enemy_type {
            let new_bullet = self.attack();
            self.bullets.push(new_bullet);
        }
    }
}

// 子弹结构体
#[derive(PartialEq)]  // 解决比较问题
struct Bullet {
    position_x: i32,
    position_y: i32,
    damage: u32,
    is_enemy_bullet: bool, // 新增标记，区分玩家和敌人的子弹
}

impl Bullet {
    fn new(player: &Player) -> Self {
        Bullet {
            position_x: player.position_x,
            position_y: player.position_y,
            damage: player.attack,
            is_enemy_bullet: false,
        }
    }

    fn new_enemy(enemy: &Enemy) -> Self {
        Bullet {
            position_x: enemy.position_x,
            position_y: enemy.position_y,
            damage: enemy.bullet_damage,
            is_enemy_bullet: true,
        }
    }

    fn move_bullet(&mut self) {
        if self.is_enemy_bullet {
            self.position_y += 1; // 敌人子弹向下移动
        } else {
            self.position_y -= 1; // 玩家子弹向上移动
        }
    }

    fn is_out_of_bounds(&self) -> bool {
        if self.is_enemy_bullet {
            self.position_y >= SCREEN_HEIGHT
        } else {
            self.position_y < 0
        }
    }

    fn hit_enemy(&self, enemy: &Enemy) -> bool {
        !self.is_enemy_bullet && self.position_x == enemy.position_x && self.position_y == enemy.position_y
    }

    fn hit_player(&self, player: &Player) -> bool {
        self.is_enemy_bullet && self.position_x == player.position_x && self.position_y == player.position_y
    }

    fn render(&self, ctx: &mut BTerm) {
        let color = if self.is_enemy_bullet { BLUE } else { RED };
        ctx.set(self.position_x, self.position_y, color, BLACK, to_cp437('*'));
    }
}

// 游戏模式
enum GameMode{
    Menu,              //主菜单
    Playing,           //游戏中 
    Equipment,         //装备界面
    End,               //游戏结束
}

// 常量
const SCREEN_WIDTH  : i32 = 80;
const SCREEN_HEIGHT : i32 = 50;
const FRAME_DURATION: f32 = 75.0;
const FRAME_COUNTDOWN   : f32 = 1000.0;
const WAVE_INTERVAL: f32 = 5000.0; // 波次间隔5秒
const STARTING_ENEMIES: u32 = 2;   // 第一波敌人数量
const ENEMY_SPAWN_X_RANGE: (i32, i32) = (5, 75); // 敌人生成x范围

// 主场景
struct State{
    mode: GameMode,         // 游戏模式
    weapon: Weapon,         // 当前武器
    bullets: Vec<Bullet>,   // 玩家子弹
    player: Player,         // 玩家
    frame_time: f32,        // 攻击频率
    countdown_time: f32,    // 倒计时
    countdown_timer: f32,   // 倒计时器
    frame_enemymove_time: f32,   // 敌人移动速度
    enemies: Vec<Enemy>,         // 敌人列表
    current_wave: u32,           // 当前波次
    wave_enemy_count: u32,        // 当前波次剩余敌人数量
    wave_timer: f32,              // 波次生成计时器
    enemy_bullet_timer: f32,      // 敌人发射子弹的计时器
}
// 关联
impl State{
    // 创建时默认主菜单模式
    fn new() -> Self{
        let initial_weapon = Weapon::MachineGun;
        State{
            mode: GameMode::Menu,
            weapon: initial_weapon.clone(),
            bullets: Vec::new(),
            player: Player::new(initial_weapon.clone()),
            frame_time: 0.0,
            countdown_time: 0.0,
            countdown_timer: FRAME_COUNTDOWN * 600.0,
            frame_enemymove_time: 0.0,
            enemies: Vec::new(),
            current_wave: 1,
            wave_enemy_count: 0,
            wave_timer: 0.0,
            enemy_bullet_timer: 0.0, // 初始化敌人发射子弹的计时器
        }
    }
    // 重新开始：游戏中模式
    fn restart(&mut self){
        self.mode = GameMode::Playing;
        self.player = Player::new(self.weapon.clone());
        self.enemies.clear();
        self.current_wave = 1;
        self.spawn_wave(self.current_wave); // 生成第一波敌人
        self.frame_time = 0.0;
        self.countdown_time = 0.0;
        self.countdown_timer = FRAME_COUNTDOWN * 600.0;
        self.frame_enemymove_time = 0.0;
        self.wave_timer = 0.0;
    }
    // 回到主菜单：主菜单模式
    fn remenu(&mut self){
        self.mode = GameMode::Menu;
    }
    // 装备界面模式
    fn equipment(&mut self){
        self.mode = GameMode::Equipment;
    }
    // 主菜单场景逻辑
    fn main_menu(&mut self, ctx: &mut BTerm){
        ctx.cls();
        ctx.print_centered(5, "Welcome to Flash Plane");
        ctx.print_centered(8, "(Q) Play Game");
        ctx.print_centered(10, "(W) Equipment");
        ctx.print_centered(12, "(E) Quit Game");

        // 主菜单按键
        if let Some(key) = ctx.key{
            match key {
                VirtualKeyCode::Q => self.restart(),
                VirtualKeyCode::W => self.equipment(),
                VirtualKeyCode::E => ctx.quitting = true,
                _ => {}
            }
        }
    }
    // 装备界面逻辑
    fn equipment_menu(&mut self, ctx: &mut BTerm){
        // 显示
        ctx.cls_bg(NAVY);
        ctx.print_centered(5, "Weapon Equipment");
        ctx.print_centered(8, "(Q) MachineGun");
        ctx.print_centered(10, "(W) Laser");
        ctx.print_centered(12, "(E) ShotGun");
        ctx.print_centered(14, "(R) Return");

        // 按键
        if let Some(key) = ctx.key{
            match key {
                VirtualKeyCode::Q => self.weapon = Weapon::MachineGun,
                VirtualKeyCode::W => self.weapon = Weapon::Laser,
                VirtualKeyCode::E => self.weapon = Weapon::ShotGun,
                VirtualKeyCode::R => self.remenu(),
                VirtualKeyCode::A => ctx.print_centered(16, self.player.attack),
                _ => {}
            }
        }
    }

    // 生成波次的核心方法
    fn spawn_wave(&mut self, wave: u32) {
        let enemy_count = STARTING_ENEMIES + (wave - 1) * 2; // 每波增加2个敌人
        self.wave_enemy_count = enemy_count;
        
        let mut rng = ThreadRng::default();  // 显式创建随机数生成器

        for _ in 0..enemy_count {
            let enemy_type = match wave {
                1 => EnemyType::Scout,
                2 => EnemyType::HeavyShip,
                3 => EnemyType::Carrier,
                // 后续波次添加更多敌人类型
                _ => EnemyType::Scout, // 默认侦察机
            };
            
            let mut enemy = Enemy::new(enemy_type);
            // 随机生成敌人初始位置（y固定在顶部，x在指定范围）

            enemy.position_x = rng.gen_range(ENEMY_SPAWN_X_RANGE.0..=ENEMY_SPAWN_X_RANGE.1);  // Rust 1.53+ 支持 ..= 语法

            enemy.position_y = 6; // 从屏幕顶部生成
            self.enemies.push(enemy);
        }
    }


    // 进入游戏：游戏中模式
    fn play(&mut self, ctx: &mut BTerm){
        ctx.cls();
        // 初始化
        self.frame_time += ctx.frame_time_ms; // 移动计时器
        self.countdown_time += ctx.frame_time_ms; // 敌人生成计时器
        self.wave_timer += ctx.frame_time_ms; // 波次计时器
        self.frame_enemymove_time += ctx.frame_time_ms; //敌人移动计时器

        // 波次生成逻辑
        if self.wave_enemy_count == 0 && self.wave_timer > WAVE_INTERVAL {
            self.current_wave += 1;
            self.spawn_wave(self.current_wave);
            self.wave_timer = 0.0;
        }
        // 倒计时
        if self.countdown_time > FRAME_COUNTDOWN{
            self.countdown_time = 0.0;
            self.countdown_timer -= FRAME_COUNTDOWN
        }
        // 攻击
        if self.frame_time > FRAME_DURATION/ self.player.attack_speed{
            self.frame_time = 0.0;
            let new_bullet = self.player.attack();
            self.bullets.push(new_bullet);
        }        

        // 玩家与敌人的碰撞检测
        for enemy in &self.enemies {
            if self.player.position_x == enemy.position_x && self.player.position_y == enemy.position_y {
                self.player.hurt(enemy.collision_damage);
            }
        }
        
        if self.frame_enemymove_time > FRAME_DURATION * 10.0{
            self.frame_enemymove_time = 0.0;
            // 1. 处理敌人移动并保留存活敌人（不处理子弹碰撞）
            self.enemies.retain_mut(|enemy| {
                enemy.move_enemy();  // 移动敌人
                enemy.health > 0    // 过滤死亡敌人
            });
        }
        

        // 2. 处理子弹移动、碰撞检测和移除（解决借用冲突）
        self.bullets.retain_mut(|bullet| {
            bullet.move_bullet();  // 移动子弹

            if bullet.is_out_of_bounds() {
                return false;  // 超出边界则移除
            }

            // 检测是否命中敌人（可变遍历敌人列表）
            let hit = self.enemies.iter_mut().any(|enemy| {
                if bullet.hit_enemy(enemy) {
                    enemy.health = enemy.health.saturating_sub(bullet.damage);
                    true  // 命中则移除子弹
                } else {
                    false
                }
            });

            !hit  // 未命中则保留子弹
        });

        // 处理敌人发射的子弹
        self.enemies.iter_mut().for_each(|enemy| {
            enemy.bullets.retain_mut(|bullet| {
                bullet.move_bullet();
                if bullet.is_out_of_bounds() {
                    return false;
                }
                // 检测是否命中玩家
                if bullet.hit_player(&self.player) {
                    self.player.hurt(bullet.damage);
                    false
                } else {
                    true
                }
            });
        });

        // 控制重型舰发射子弹的频率
        if self.enemy_bullet_timer > FRAME_COUNTDOWN { // 每1秒发射一次子弹
            self.enemy_bullet_timer = 0.0;
            self.enemies.iter_mut().for_each(|enemy| {
                enemy.shoot();
            });
        }
        
        // 更新波次剩余敌人数量
        self.wave_enemy_count = self.enemies.len() as u32;

        // 操作按键
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::W => self.player.move_player('W'),
                VirtualKeyCode::A => self.player.move_player('A'),
                VirtualKeyCode::S => self.player.move_player('S'),
                VirtualKeyCode::D => self.player.move_player('D'),
                VirtualKeyCode::R => self.restart(),
                VirtualKeyCode::E => self.remenu(),
                _ => {}
            }
        }

        // 显示模块
        // 渲染
        self.player.render(ctx);
        for mut enemy in &mut self.enemies {
            enemy.render(ctx); // 渲染敌人和血条
        }
        for bullet in &self.bullets {
            bullet.render(ctx);
        }
        // 渲染敌人发射的子弹
        for enemy in &self.enemies {
            for bullet in &enemy.bullets {
                bullet.render(ctx);
            }
        }
        // 信息面板
        ctx.print_centered(0,format!("{}:{}",(self.countdown_timer as i32)/60000, (self.countdown_timer as i32)%60000/1000));
        ctx.print(60, 0, format!("atk:{}    speed:{}",self.player.attack,self.player.attack_speed));
        ctx.print_centered(3,"Press WASD to Move     Press R to restart     Press E to remenu");
        for i in 0..SCREEN_WIDTH{
            ctx.print(i,5,"-");
        }

        if self.player.health <= 0{
            self.mode = GameMode::End;
        }
        
    }
    // 死亡：死亡后逻辑
    fn dead(&mut self, ctx: &mut BTerm){
        ctx.cls();
        ctx.print_centered(5, "You are dead!");
        ctx.print_centered(8, "(Q) Play Again");
        ctx.print_centered(10, "(W) Return Menu");

        if let Some(key) = ctx.key{
            match key {
                VirtualKeyCode::Q => self.restart(),
                VirtualKeyCode::W => self.remenu(),
                _ => {}
            }
        }
    }
}

// 游戏每帧判断模式
impl GameState for State{
    fn tick(&mut self, ctx: &mut BTerm){
        match self.mode{
            GameMode::Menu => self.main_menu(ctx),
            GameMode::Equipment => self.equipment_menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::End => self.dead(ctx)
        }
    }
}

fn main() -> BError {
    //游戏窗口
    let context: BTerm = BTermBuilder::simple80x50()
        .with_title("Flash Plane")
        .build()?;

    // 游戏主循环
    main_loop(context, State::new())
}    