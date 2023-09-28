use std::{
    ops::{Add, Mul},
    sync::Arc,
};

use bevy::prelude::{
    shape::{Cylinder, Quad},
    system_adapter::new,
    *,
};
use bevy_egui::{
    egui::{self, Slider, Ui},
    EguiContexts,
};
use static_math::{self, DualQuaternion, Quaternion};

#[derive(Default, Clone)]
struct DualQuatCtrls {
    theta: f32,
    rot: Vec3,
    rigid_body_comps: Vec3,
}

// This struct stores the values for the sliders, so that they persist between frames
// As EGUI is immediate mode, we have to maintain the state of the GUI ourselves
#[derive(Resource, Default, Clone)]
struct UiState {
    dual_quat1: DualQuatCtrls,
    dual_quat2: DualQuatCtrls,
}

#[derive(Resource, Default, Clone)]
struct JointTrans {
    dual_quat: Vec<DualQuaternion<f32>>,
}

#[derive(Component, Default, Debug)]
struct Transformable {
    node_transform: Transform,
    id: usize,
}

impl Transformable {
    fn with_id(mut self, id: usize) -> Self {
        self.id = id;
        self
    }
}

impl From<Transform> for Transformable {
    fn from(node_transform: Transform) -> Self {
        Self {
            node_transform,
            id: 0,
        }
    }
}

pub trait InternalFrom<T>: Sized {
    fn ext_from(value: T) -> Self;
}

impl InternalFrom<Quat> for static_math::Quaternion<f32> {
    fn ext_from(quat: Quat) -> Self {
        static_math::Quaternion::new_from(quat.w, quat.x, quat.y, quat.z)
    }
}

impl InternalFrom<static_math::Quaternion<f32>> for Quat {
    fn ext_from(quaternion: static_math::Quaternion<f32>) -> Self {
        let real: f32 = quaternion.real();
        let imaginary: static_math::V3<f32> = quaternion.imag();
        Quat::from_xyzw(imaginary[0], imaginary[1], imaginary[2], real)
    }
}

// impl InternalFrom<static_math::matrix3x3::M33<f32>> for Mat3 {
//     fn ext_from(static_mat3: static_math::matrix3x3::M33<f32>) -> Self {
//         let s = static_mat3.get_rows();
//         let arr: [f32; 9] = [
//             s[0][0], s[0][1], s[0][2], s[1][0], s[1][1], s[1][2], s[2][0], s[2][1], s[2][2],
//         ];
//         // TODO: check if we need to transpose
//         Self::from_cols_array(&arr)
//     }
// }

// trait Vec3Ext {
//     fn mul_all(&self, rhs: f32) -> Vec3;
// }

// impl Vec3Ext for Vec3 {
//     fn mul_all(&self, rhs: f32) -> Vec3 {
//         Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
//     }
}
// impl InternalFrom<static_math::matrix3x3::M33<f32>> for Mat3 {
//     fn ext_from(static_mat3: static_math::matrix3x3::M33<f32>) -> Self {
//         let s = static_mat3.get_rows();
//         let arr: [f32; 9] = [
//             s[0][0], s[0][1], s[0][2], s[1][0], s[1][1], s[1][2], s[2][0], s[2][1], s[2][2],
//         ];
//         // TODO: check if we need to transpose
//         Self::from_cols_array(&arr)
//     }
// }

// impl InternalFrom<Mat3> for static_math::matrix3x3::M33<f32> {
//     fn ext_from(m: Mat3) -> Self {
//         let m = m.to_cols_array_2d();
//         Self::new([m[0], m[1], m[2]])
//     }
// }

// Main entrypoint
fn main() {
    // App entrypoint
    App::new()
        // Bevy plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Quaternion Forward Kinematics Demo".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(bevy_egui::EguiPlugin)
        // Systems (functions that are called at regular intervals)
        .add_systems(Startup, setup)
        .add_systems(Update, transform_ui)
        // Resources (live data that can be accessed from any system)
        .init_resource::<UiState>()
        .init_resource::<JointTrans>()
        .run(); // Event loop etc occurs here
}

trait NewMesh {
    fn new_box(x: f32, y: f32, z: f32) -> Mesh;
    fn new_typical_cylinder(radius: f32, height: f32) -> Mesh;
    fn new_cylinder(radius: f32, height: f32, resolution: u32, segments: u32) -> Mesh;
}

impl NewMesh for Mesh {
    fn new_box(x: f32, y: f32, z: f32) -> Mesh {
        Mesh::from(shape::Box::new(x, y, z))
    }

    fn new_typical_cylinder(radius: f32, height: f32) -> Mesh {
        Mesh::from(shape::Cylinder {
            radius,
            height,
            resolution: 64,
            segments: 1,
        })
    }

    fn new_cylinder(radius: f32, height: f32, resolution: u32, segments: u32) -> Mesh {
        Mesh::from(shape::Cylinder {
            radius,
            height,
            resolution,
            segments,
        })
    }
}

// Setup basic facilities
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut joint_trans: ResMut<JointTrans>,
) {
    for i in 0..2 {
        let material = materials.add(Color::WHITE.into());
        let mesh_base = meshes.add(Mesh::new_typical_cylinder(1.5, 1.));
        let mesh_middle = meshes.add(Mesh::new_typical_cylinder(0.5, 2.));
        let mesh_arm = meshes.add(Mesh::new_box(1.0, 0.9, 4.0));
        let transform = Transform::from_translation(Vec3::ZERO);
        let arm_transform = Transform::from_xyz(0.0, 0.0, -2.0);
        let mesh_base = meshes.add(Mesh::new_typical_cylinder(1.5, 1.));
        let mesh_middle = meshes.add(Mesh::new_typical_cylinder(0.5, 2.));
        let mesh_arm = meshes.add(Mesh::new_box(1.0, 0.9, 4.0));

        commands.spawn((
            PbrBundle {
                mesh: mesh_base,
                material: material.clone(),
                transform,
                ..default()
            },
            Transformable::default().with_id(i),
        ));

        commands.spawn((
            PbrBundle {
                mesh: mesh_middle,
                material: material.clone(),
                transform,
                ..default()
            },
            Transformable::default().with_id(i),
        ));

        commands.spawn((
            PbrBundle {
                mesh: mesh_arm,
                material: material.clone(),
                transform: arm_transform,
                ..default()
            },
            Transformable::from(arm_transform).with_id(i),
        ));
    }

    // Camera is necessary to render anything
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Spawn a light so that it's easier to see the cube
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::ONE * 3.0),
        ..default()
    });
}

trait TensorProdVec3 {
    fn tensor_prod(&self, rhs: Self) -> Mat3;
}

impl TensorProdVec3 for Vec3 {
    fn tensor_prod(&self, rhs: Self) -> Mat3 {
        Mat3 {
            x_axis: self.x * rhs,
            y_axis: self.y * rhs,
            z_axis: self.z * rhs,
        }
    }
}

// This is where the transform happens
fn transform_ui(
    mut transformables: Query<(&mut Transform, &mut Transformable)>,
    mut ui_state: ResMut<UiState>,
    mut joint_trans: ResMut<JointTrans>,
    mut ctx: EguiContexts,
) {
    // A wrapper function for creating a slider with common settings,
    // e.g. range, clamp, step_by, etc
    fn common_slider<'a>(value: &'a mut f32, text: &str) -> Slider<'a> {
        Slider::new(value, -10.0..=10.0)
            .text(text)
            .clamp_to_range(false)
            .drag_value_speed(0.01)
            .step_by(0.01)
    }

    let dual_quat_sliders = |ui: &mut Ui, dq_ctrls: &mut DualQuatCtrls| {
        ui.add(common_slider(&mut dq_ctrls.theta, "theta"));
        ui.add(common_slider(&mut dq_ctrls.rot.x, "pitch axis"));
        ui.add(common_slider(&mut dq_ctrls.rot.y, "yaw axis"));
        ui.add(common_slider(&mut dq_ctrls.rot.z, "roll axis"));
        ui.add(common_slider(
            &mut dq_ctrls.rigid_body_comps.x,
            "Rigid Body X",
        ));
        ui.add(common_slider(
            &mut dq_ctrls.rigid_body_comps.y,
            "Rigid Body Y",
        ));
        ui.add(common_slider(
            &mut dq_ctrls.rigid_body_comps.z,
            "Rigid Body Z",
        ));
    };
    // The floating EGUI window
    egui::Window::new("Quaternion control").show(ctx.ctx_mut(), |ui| {
        // Note that the code inside this block is part of a closure, similar to lambdas in Python.

        // Slider width style
        ui.style_mut().spacing.slider_width = 450.0;
        // Sliders are added here, passed mutable access to the variables storing their states
        dual_quat_sliders(ui, &mut ui_state.dual_quat1);
        dual_quat_sliders(ui, &mut ui_state.dual_quat2);
    });

    let dq_from_ctrls = |ctrls: &DualQuatCtrls| {
        let theta = ctrls.theta;

        let real_quat = ((theta * ctrls.rot) / 2.0).exp();
        let real_quat_w = (-theta / 2.).exp();
        let imag_quat = (0.5 * ctrls.rigid_body_comps) * (theta / 2.0).exp();

        DualQuaternion::new_from_array([
            // real quat refers to the roll/pitch/yaw of the axis.
            real_quat.x,
            real_quat.y,
            real_quat.z,
            // real quat w is how big of a turn after you get the axis to the new location.
            real_quat_w,
            // This is translation.
            imag_quat.x,
            imag_quat.y,
            imag_quat.z,
        ])
    };

    let dq1 = dq_from_ctrls(&ui_state.dual_quat1);
    let dq2 = dq_from_ctrls(&ui_state.dual_quat2);

    let base_dual_quat = DualQuaternion::<f32>::one();
    // Iterate over all transformables

    for (mut transform, transformable) in &mut transformables {
        let dq = match transformable.id {
            0 => base_dual_quat * dq1,
            1 => base_dual_quat * dq1 * dq2,
            _ => {
                panic!("wrong id gfy");
            }
        };
        let base_transform = Transform {
            rotation: Quat::ext_from(dq.real()).normalize(),
            translation: Quat::ext_from(dq.dual()).xyz(),
            scale: Vec3::ONE,
        };

        *transform = base_transform * transformable.node_transform;
    }
}
