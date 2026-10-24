// =============================================================================
// Bevy 0.20 公式サンプル: `shader_material.rs` & `custom_material.wesl` 詳細日本語解説版
// =============================================================================
// 【概要】
// このファイルは、Bevyエンジンで独自シェーダー（WESL）とカスタムマテリアル（`CustomMaterial`）を
// 定義し、3Dメッシュ（立方体）に適用して描画する最小構成の完全解説コードです。
//
// 1. Rust側の設定（マテリアル構造体、`AsBindGroup`属性、`MaterialPlugin`の登録）
// 2. WESLシェーダー側の記述（バインドグループ、テクスチャサンプリング、フラグメントシェーダー）
// の両方をインラインコメントで徹底解説しています。
// =============================================================================

use bevy::{
    prelude::*,                             // Bevyの基本型（Commands, Mesh, Transform, Camera3dなど）を一括インポート
    reflect::TypePath,                      // リフレクション用：型パス識別を提供するトレイト（`#[derive(TypePath)]`に必要）
    render::render_resource::AsBindGroup,   // GPUバインドグループ（Uniform, Texture, Sampler）を自動生成するDeriveマクロ
    shader::ShaderRef,                      // シェーダーファイルパス（`"shaders/custom_material.wesl"`）を保持・参照する型
};
use bevy::{
    render::{RenderPlugin, settings::{WgpuSettings, RenderCreation, InstanceFlags}},
};

/// カスタムシェーダー（WESL）アセットファイルのパスを定数として定義
/// Bevyのアセットローダーはデフォルトで `assets/` ディレクトリ配下を参照します
const SHADER_ASSET_PATH: &str = "shaders/custom_material.wesl";

fn main() {
    App::new()
        // DefaultPlugins: レンダラー、ウィンドウ、入力管理などの標準基本機能を読み込み
        .add_plugins((
            DefaultPlugins
            //.set(ImagePlugin::default_nearest())
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(Box::new(
                    WgpuSettings {
                        instance_flags: InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER,
                        ..default()
                    },
                )),
                ..default()
            }),
            // MaterialPlugin::<CustomMaterial>::default():
            // 独自で定義した `CustomMaterial` 用のレンダーパイプライン・シェーダーコンパイル機構を初期化して登録。
            // ※これを忘れると「No pipeline for material」というエラーで描画されません。
            MaterialPlugin::<CustomMaterial>::default(),
        ))
        // アプリ起動（Startup）時に1回だけ実行されるセットアップシステムを登録
        .add_systems(Startup, setup)
        // メインループ（ゲームループ）を開始
        .run();
}

/// 3Dシーンの初期化システム（カメラ・ライティング・カスタムマテリアル適用メッシュの生成）
fn setup(
    mut commands: Commands,                         // ワールドにEntityを生成（spawn）するためのコマンドバッファ
    mut meshes: ResMut<Assets<Mesh>>,               // 3Dメッシュ（形状）アセットバッファの可変参照
    mut materials: ResMut<Assets<CustomMaterial>>,  // 自作カスタムマテリアルアセットバッファの可変参照
    asset_server: Res<AssetServer>,                 // テクスチャ画像などの外部アセットを読み込むサーバー
) {
    // -------------------------------------------------------------------------
    // 1. 立方体（Cube）Entityの生成
    // -------------------------------------------------------------------------
    commands.spawn((
        // Mesh3d: 描画する3Dメッシュのハンドル（Handle<Mesh>）を保持するコンポーネント。
        // Cuboid::default() で1.0x1.0x1.0の立方体メッシュを生成し、アセットとして登録。
        Mesh3d(meshes.add(Cuboid::default())),

        // MeshMaterial3d: メッシュに適用するマテリアルのハンドル（Handle<CustomMaterial>）を保持。
        // `CustomMaterial` のインスタンスを生成してアセット登録します。
        MeshMaterial3d(materials.add(CustomMaterial {
            // uniform(0) に渡されるベースカラー（青色）
            color: LinearRgba::BLUE,
            // texture(1) & sampler(2) に渡されるテクスチャ画像（Bevyロゴアイコン）の非同期読み込み
            color_texture: Some(asset_server.load("branding/papa.png")),
            // アルファ合成モード（Blend: 透過対応）
            alpha_mode: AlphaMode::Blend,
        })),

        // Transform: 3D空間内の位置情報（Y軸方向に+0.5移動）
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // -------------------------------------------------------------------------
    // 2. 3DカメラEntityの生成
    // -------------------------------------------------------------------------
    commands.spawn((
        Camera3d::default(),
        // 座標 (-2.0, 2.5, 5.0) から 原点 (0.0, 0.0, 0.0) を見つめるカメラ視点
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// =============================================================================
// カスタムマテリアル構造体の定義（Rust ⇄ GPU 間のデータ橋渡し）
// =============================================================================

/// GPUへデータを送るためのマテリアル構造体
///
/// 【使用しているDeriveマクロの解説】
/// - `Asset`: Bevyのアセットシステム（`Assets<CustomMaterial>`）で管理可能にする
/// - `TypePath`: 型の名前空間パスを取得可能にする（アセット識別用）
/// - `AsBindGroup`: 構造体のフィールドを解析し、GPUの BindGroup / BindGroupLayout を自動構築する最重要マクロ！
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {
    /// #[uniform(0)]
    /// - このフィールド（LinearRgba: 16バイトのvec4<f32>）を GPUの BindGroup 0（グループ2内）の binding(0) にUniformバッファとして割り当てます。
    #[uniform(0)]
    color: LinearRgba,

    /// #[texture(1)] & #[sampler(2)]
    /// - 画像ハンドル（`Option<Handle<Image>>`）を GPUの binding(1) (2Dテクスチャ) と binding(2) (サンプラー) に自動バインドします。
    #[texture(1)]
    #[sampler(2)]
    color_texture: Option<Handle<Image>>,

    /// アルファ透過モード（Blend, Opaque, Maskなど）。
    /// ※これはGPUのパイプライン設定（ブレンディング等）で参照され、シェーダーユニフォームバッファには含まれません。
    alpha_mode: AlphaMode,
}

/// `Material` トレイトの実装
///
/// Bevyの `Material` トレイトはデフォルト実装が充実しているため、オーバーライドしたい機能のみを記述します。
impl Material for CustomMaterial {
    /// 使用するフラグメントシェーダー（WESL）のパスを返す
    /// 独自頂点シェーダーを指定したい場合は `fn vertex_shader()` もオーバーライドします。
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    /// マテリアルの透過設定（Blendモードなど）を判定するAPI
    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}


/* =============================================================================
 * 【WESLシェーダーコード】 `assets/shaders/custom_material.wesl`
 * =============================================================================
 *
 * Bevy 0.20では、WebGPU標準のWESL（WebGPU Extended Shading Language）が公式採用されています。
 * Rust側の `AsBindGroup` で定義した属性に対応するバインディングを定義します。
 *
 * -----------------------------------------------------------------------------
 * // WESLモジュールのインポート（Bevy標準の頂点出力構造体を取り込み）
 * import bevy_pbr::render::forward_io::VertexOutput;
 *
 * // 相対パスでの別シェーダーモジュール（定数など）のインポート例
 * import super::custom_material_import::COLOR_MULTIPLIER;
 *
 * // -------------------------------------------------------------------------
 * // バインドグループの定義
 * // -------------------------------------------------------------------------
 * // ※重要：Bevyでは Group 0（カメラ・ビュー情報）、Group 1（メッシュ変形行列）がエンジン側で予約されています。
 * // したがって、マテリアル固有のデータは必ず `@group(constants::MATERIAL_BIND_GROUP)` （Group 2）になります！
 *
 * // Rust側: #[uniform(0)] color: LinearRgba -> binding(0)
 * @group(constants::MATERIAL_BIND_GROUP) @binding(0)
 * var<uniform> material_color: vec4<f32>;
 *
 * // Rust側: #[texture(1)] color_texture -> binding(1)
 * @group(constants::MATERIAL_BIND_GROUP) @binding(1)
 * var material_color_texture: texture_2d<f32>;
 *
 * // Rust側: #[sampler(2)] color_texture -> binding(2)
 * @group(constants::MATERIAL_BIND_GROUP) @binding(2)
 * var material_color_sampler: sampler;
 *
 * // -------------------------------------------------------------------------
 * // フラグメントシェーダー関数（ピクセルごとの色出力）
 * // -------------------------------------------------------------------------
 * @fragment
 * fn fragment(
 *     mesh: VertexOutput, // 頂点シェーダーから補間されて渡されるUV座標や法線ベクトル
 * ) -> @location(0) vec4<f32> {
 *     // 1. テクスチャ画像（binding 1）をサンプラー（binding 2）とUV座標（mesh.uv）でサンプリング（色取得）
 *     let sampled_color = textureSample(material_color_texture, material_color_sampler, mesh.uv);
 *
 *     // 2. マテリアル色 (material_color) × テクスチャ色 × 乗数 (COLOR_MULTIPLIER) を計算して最終ピクセル色を返す
 *     return material_color * sampled_color * COLOR_MULTIPLIER;
 * }
 * =============================================================================
 */
