const run_freq: any = 500.0;
const leg_pos: any = 0.6;

fn vx_sin(context: any) {
    if let Jump = context.action {
        (context.timer / 50.0).min(4.0)
    } else {
        (tau * (context.timer / run_freq)).sin() * context.vx.abs()
    }
}

fn body_offset(context: any) {
    vx_sin(context).abs() * 0.04
}

fn vector_theta(vector: any) {
    vector.1.atan2(vector.0)
}

fn vector_cos(vector: any) {
    vector_theta(vector).cos()
}

fn female(context: any) {
    let vx_sin = vx_sin(context);
    let body_offset = body_offset(context);
    let bones = vec![
        Bone {
            sprite: "female_arm.png",
            pivot_offset: (0, -0.3),
            offset: (vx_sin * 0.01, -0.2),
            flip: context.facing == Right,
            rotation: vx_sin * -10.0,
        },
        // back leg
        Bone {
            sprite: "female_leg.png",
            offset: (vx_sin * -0.05, leg_pos + body_offset * -1),
            pivot_offset: (0, -0.3),
            rotation: vx_sin * 5.0,
        },
        // front leg
        Bone {
            sprite: "female_leg.png",
            offset: (vx_sin * 0.05, leg_pos + body_offset * -1),
            pivot_offset: (0, -0.3),
            rotation: vx_sin * -5.0,
        },
        // body
        Bone {
            sprite: "female_body.png",
            flip: context.facing == Right,
            offset: (0.0, 0.0),
            rotation: 0.0,
            pivot_offset: (0.0, 0.0),
        },
        Bone {
            sprite: "female_head.png",
            flip: if let Throw(vec) = context.arm_action {
                vec.0 > 0
            } else {
                context.facing == Right
            },
            offset: (0.0, -0.8),
            pivot_offset: (-0.1, 0.0),
            rotation: 0.0,
        },
    ];
    if let Throw(throw) = context.arm_action {
        bones.push(Bone {
            sprite: "arrow_thinner.png",
            flip: true,
            offset: (
                context.throw_theta.cos() * 0.3,
                -0.2 + 0.3 * context.throw_theta.sin(),
            ),
            // pivot_offset: (Plain(0), Plain(0.0)),
            pivot_offset: (0.0, 0.5 + -0.02 * context.throw_mag),
            scale: 1.5,
            rotation: if (context.throw_vx > 0.0) {
                ((context.throw_theta / pi) * 180.0) + 180.0 + -90.0
            } else {
                90.0 + ((context.throw_theta / pi) * 180.0)
            },
        })
    };
    if let Some(point_theta) = context.point_theta {
        bones.push(Bone {
            sprite: "female_arm.png",
            flip: context.facing == Right,
            offset: ((0.0), (-0.02)),
            pivot_offset: ((0), (-0.3)),
            rotation: (((point_theta / pi) * (180)) + (90)),
        })
    } else {
        bones.push(match context.arm_action {
            None => Bone {
                sprite: "female_arm.png",
                flip: context.facing == Right,
                offset: ((vx_sin * (-0.01)), (-0.02)),
                pivot_offset: ((0.0), (-0.3)),
                rotation: (vx_sin * (10.0)),
            },
            ThrowArm => Bone {
                sprite: "female_arm.png",
                flip: context.facing == Right,
                offset: ((0.0), (-0.2)),
                pivot_offset: ((0.00), (-0.3)),
                rotation: if (context.throw_vx > (0.0)) {
                    (((context.throw_theta / pi) * (180.0)) + (180.0))
                } else {
                    ((context.throw_theta / pi) * (180.0))
                },
            },
            Throw => Bone {
                sprite: "female_arm.png",
                flip: context.facing == Right,
                offset: ((0), (-0.2)),
                pivot_offset: ((0), ((-0.3) + ((0.02) * context.throw_mag))),
                rotation: if (context.throw_vx > 0.0) {
                    context.throw_theta / pi * 180.0 + 270.0
                } else {
                    context.throw_theta / pi * 180.0 + -90
                },
            }
        })
    };

    if context.arm_action == Throw {
        bones.push(Bone {
            sprite: "bow.png",
            flip: true,
            offset: (
                (0.3) * context.throw_theta.cos(),
                (-0.2) + ((0.3) * context.throw_theta.sin()),
            ),
            pivot_offset: ((0.0), (0.0)),
            scale: (1.5),
            rotation: if (context.throw_vx > 0.0) {
                (((context.throw_theta / pi) * (180.0)) + (180.0)) + (40)
            } else {
                ((-135.0) + ((context.throw_theta / pi) * (180.0)))
            },
        })
    };

    Skeleton {
        shape: Capsule {
            width: 0.1,
            height: 0.1,
        },
        scale: 0.3,
        offset: (0, body_offset(context)),
        bones: bones,
    }
}
