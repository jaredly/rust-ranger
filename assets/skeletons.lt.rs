const run_freq: any = 500.0;
const leg_pos: any = 0.6;

fn vx_sin(context: any, velocity: any) {
    if let Jump = context.action {
        (context.timer / 50.0).min(4.0)
    } else {
        (tau * (context.timer / run_freq)).sin() * velocity.0.abs()
    }
}

fn body_offset(context: any, velocity: any) {
    vx_sin(context, velocity).abs() * 0.04
}

fn vector_mag(vector: any) {
    (vector.0 * vector.0 + vector.1 * vector.1).sqrt()
}

fn vector_theta(vector: any) {
    vector.1.atan2(vector.0)
}

fn vector_cos(vector: any) {
    vector_theta(vector).cos()
}

// TODO actually maybe function arguments should be pass by reference.....
fn arm_position(arm_action: any, flip: any) {
    // log(arm_action);
    match arm_action {
        None => (
            ((vx_sin * (-0.01)), (-0.02)),
            ((0.0), (-0.3)),
            (vx_sin * (10.0)),
        ),
        Bow(vec) => (
            ((0.0), (-0.2)),
            ((0.0), (-0.3)),
            if (vec.0 > (0.0)) {
                (((vector_theta(vec) / pi) * (180.0)) + (180.0))
            } else {
                ((vector_theta(vec) / pi) * (180.0))
            },
        ),
        Throw(vec) => (
            ((0.0), (-0.2)),
            ((0.0), ((-0.3) + ((0.02) * vector_mag(vec.clone())))),
            if (vec.0 > 0.0) {
                vector_theta(vec) / pi * 180.0 + 270.0
            } else {
                vector_theta(vec) / pi * 180.0 + -90.0
            },
        ),
        Swing {position, forward, object, direction} => (
            (0.0, -0.02),
            (0.0, -0.3),
            if flip {
                (-180.0 + position * 140.0 + match direction {
                    Up => -90.0,
                    Down => 90.0,
                    Forward => 0.0
                })
            } else {
                (180.0 - position * 140.0 + match direction {
                    Up => 90.0,
                    Down => -90.0,
                    Forward => 0.0
                })
            }
        )
    }
}

fn tool_tip(arm_action: any, facing: any) {
    let flip = facing == Right;
    let (offset, pivot_offset, rotation) = arm_position(arm_action, flip.clone());
    let rotation = rotation + if flip.clone() { -80.0 } else { -50.0 };
    let rotation = rotation * pi / 180.0;
    let pivot_offset = (-0.8, 0.4);
    let angle = pivot_offset.1.atan2(pivot_offset.0);
    let mag = vector_mag(pivot_offset.clone());
    (
        offset.0 + mag * (angle + rotation).cos() * 0.5,
        offset.1 + mag * (angle + rotation).sin() * 0.5,
    )
}

fn female(context: any, velocity: any) {
    let vx_sin = vx_sin(context.clone(), velocity.clone());
    let body_offset = body_offset(context.clone(), velocity.clone());
    let bones = vec![
        Bone {
            sprite: "female_arm.png",
            pivot_offset: (0.0, -0.3),
            offset: (vx_sin * 0.01, -0.2),
            flip: context.facing == Right,
            rotation: vx_sin * -10.0,
        },
        // back leg
        Bone {
            sprite: "female_leg.png",
            offset: (vx_sin * -0.05, leg_pos + body_offset * -1.0),
            pivot_offset: (0.0, -0.3),
            rotation: vx_sin * 5.0,
            flip: context.facing == Right,
        },
        // front leg
        Bone {
            sprite: "female_leg.png",
            offset: (vx_sin * 0.05, leg_pos + body_offset * -1.0),
            pivot_offset: (0.0, -0.3),
            rotation: vx_sin * -5.0,
            flip: context.facing == Right,
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
                vec.0 > 0.0
            } else {
                context.facing == Right
            },
            offset: (0.0, -0.8),
            pivot_offset: (-0.1, 0.0),
            rotation: 0.0,
        },
    ];
    if let Throw(throw) = context.arm_action {
        let theta = vector_theta(throw.clone());
        bones.push(Bone {
            sprite: "arrow_thinner.png",
            flip: true,
            offset: (
                theta.cos() * 0.3,
                -0.2 + 0.3 * theta.sin(),
            ),
            pivot_offset: (0.0, 0.5 + -0.02 * vector_mag(throw.clone())),
            scale: 1.5,
            rotation: if (throw.0 > 0.0) {
                ((theta / pi) * 180.0) + 180.0 + -90.0
            } else {
                90.0 + ((theta / pi) * 180.0)
            },
        })
    };
    if let Some(vec) = context.pointing {
        bones.push(Bone {
            sprite: "female_arm.png",
            flip: context.facing == Right,
            offset: ((0.0), (-0.02)),
            pivot_offset: ((0), (-0.3)),
            rotation: (((vector_theta(vec) / pi) * (180.0)) + (90.0)),
        })
    } else {
        let (offset, pivot_offset, rotation) = arm_position(context.arm_action, context.facing == Right);
        if let Swing {object: object} = context.arm_action {
            bones.push(Bone {
                sprite: object,
                // flip: false,
                flip: context.facing == Left,
                offset: (0.0,0.0),
                pivot_offset: (-0.8, 0.3),
                // offset: (0.7, 0.7),
                rotation: rotation + if context.facing == Right { 90.0 } else { -90.0 },

                // pivot_offset: pivot_offset,
                // offset: offset,
                scale: 1.3,
            })
        };
        bones.push(Bone {
            sprite: "female_arm.png",
            flip: context.facing == Right,
            offset: offset.clone(),
            pivot_offset: pivot_offset.clone(),
            rotation: rotation.clone(),
        })
    };

    if let Throw(vec) = context.arm_action {
        let theta = vector_theta(vec.clone());
        bones.push(Bone {
            sprite: "bow.png",
            flip: true,
            offset: (
                (0.3) * theta.cos(),
                (-0.2) + ((0.3) * theta.sin()),
            ),
            pivot_offset: ((0.0), (0.0)),
            scale: (1.5),
            rotation: if (vec.0 > 0.0) {
                (((theta / pi) * (180.0)) + (180.0)) + (40.0)
            } else {
                ((-135.0) + ((theta/ pi) * (180.0)))
            },
        })
    };

    Skeleton {
        shape: Capsule {
            width: 0.1,
            height: 0.1,
        },
        scale: 0.3,
        offset: (0.0, body_offset(context.clone(), velocity.clone())),
        bones: bones,
    }
}
