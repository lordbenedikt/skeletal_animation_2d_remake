#[cfg(test)]
mod tests {
    use super::super::*;
    use assert::*;
    use std::f32::consts::PI;

    #[test]
    fn get_gl_transform_works() {
        let mut chain = vec![];

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
        chain.push(t3);
        chain.push(t2);
        chain.push(t1);

        assert_transform_eq(&get_gl_transform(1, &chain), &combined_transform(&t1, &t2));

        assert_transform_eq(
            &get_gl_transform(0, &chain),
            &combined_transform(&t1, &combined_transform(&t2, &t3)),
        );
    }

    #[test]
    fn get_tip_works() {
        let transform = Transform {
            translation: Vec3::new(13.0, 40.0, -20.0),
            scale: Vec3::new(2.0, 2.0, 2.0),
            ..Default::default()
        };

        assert_eq!(&get_tip(&transform), &Vec3::new(13.0, 42.0, -20.0),);
    }

    #[test]
    fn get_tip_chain_works() {
        let t1 = Transform {
            translation: Vec3::new(13.0, 40.0, -20.0),
            scale: Vec3::new(2.0, 2.0, 2.0),
            ..Default::default()
        };
        let t2 = Transform {
            translation: Vec3::new(0.0, 1.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            rotation: Quat::from_rotation_z(PI / 2.0),
        };
        let t3 = Transform {
            translation: Vec3::new(0.0, 1.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            ..Default::default()
        };
        let chain = vec![t3, t2, t1];

        assert_eq!(&get_tip_chain(&chain), &Vec3::new(9.0, 42.0, -20.0));
    }
}
