#[cfg(test)]
mod tests {
    use super::super::*;
    use std::f32::consts::PI;
    use assert::*;

    #[test]
    fn combined_transform_translation() {
        let t1 = Transform::from_translation(Vec3::new(220.0, 33.0, 12.0));
        let t2 = Transform::from_translation(Vec3::new(34.0, 2.7, 0.5));

        assert_eq!(
            combined_transform(&t1, &t2),
            Transform::from_translation(Vec3::new(254.0, 35.7, 12.5))
        );
    }

    #[test]
    fn combined_transform_scale() {
        let t1 = Transform::from_scale(Vec3::new(1.0, 2.5, 3.0));
        let t2 = Transform::from_scale(Vec3::new(0.1, 2.0, 2.5));

        assert_eq!(
            combined_transform(&t1, &t2),
            Transform::from_scale(Vec3::new(0.1, 5.0, 7.5))
        );
    }

    #[test]
    fn combined_transform_rotation() {
        let t1 = Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, PI / 2.0, 0.0, 0.0));
        let t2 = Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0.0, PI / 2.0, 0.0));
        let t3 = Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, PI / 2.0));

        // The two angles are equivalent
        let euler_1 =
            Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 3.0 * PI, 2.0 * PI, 0.0));
        let euler_2 = Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, PI, 0.0, -2.0 * PI));

        let t_combined_1 =
            Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, PI, PI / 2.0, 0.0));
        let t_combined_2 = Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            PI / 2.0,
            PI / 2.0,
            PI / 2.0,
        ));

        assert_transform_eq(
            &combined_transform(&combined_transform(&t1, &t2), &t3),
            &t_combined_1,
        );
        assert_transform_eq(
            &combined_transform(&combined_transform(&t1, &t2), &t3),
            &t_combined_2,
        );
        assert_transform_eq(&euler_1, &euler_2);
    }

    #[test]
    fn to_global_and_back_to_local_transform() {
        let t1 = Transform {
            translation: Vec3::new(13.0, 40.0, -20.0),
            scale: Vec3::new(1.0, 3.0, 1.0),
            rotation: Quat::from_rotation_z(PI),
        };
        let t2 = Transform {
            translation: Vec3::new(30.0, 10.0, 200.0),
            scale: Vec3::new(0.5, 1.0, 1.0),
            rotation: Quat::from_rotation_z(1.5 * PI),
        };
        let t3 = Transform {
            translation: Vec3::new(5.0, -3.0, 100.0),
            scale: Vec3::new(3.0, 1.5, 1.0),
            rotation: Quat::from_rotation_z(-2.0 * PI),
        };

        let t2_global = combined_transform(&t1, &t2);
        let t3_global = combined_transform(&combined_transform(&t1, &t2), &t3);

        assert_transform_eq(&get_relative_transform(&t2_global, &t3_global), &t3)
    }
}
