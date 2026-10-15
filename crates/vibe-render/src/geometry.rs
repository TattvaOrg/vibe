/// Mathematical squircle and perimeter mapping utilities.

/// Compute a point along a rounded rectangle perimeter given normalized parameter t in [0.0, 1.0].
pub fn squircle_perimeter_point(
    t: f32,
    width: f32,
    height: f32,
    corner_radius: f32,
    inset: f32,
) -> (f32, f32) {
    let w = width - 2.0 * inset;
    let h = height - 2.0 * inset;
    let r = corner_radius.min(w * 0.5).min(h * 0.5);

    // Approximate perimeter segments
    let straight_w = (w - 2.0 * r).max(0.0);
    let straight_h = (h - 2.0 * r).max(0.0);
    let corner_arc = std::f32::consts::PI * 0.5 * r;

    let total_len = 2.0 * straight_w + 2.0 * straight_h + 4.0 * corner_arc;
    let dist = t.fract() * total_len;

    // Map distance along top, top-right corner, right, bottom-right, bottom, bottom-left, left, top-left
    let mut accumulated = 0.0;

    // 1. Top straight
    if dist < accumulated + straight_w {
        let frac = (dist - accumulated) / straight_w;
        return (inset + r + frac * straight_w, inset);
    }
    accumulated += straight_w;

    // 2. Top-right corner arc
    if dist < accumulated + corner_arc {
        let angle = ((dist - accumulated) / corner_arc) * (std::f32::consts::PI * 0.5);
        return (width - inset - r + r * angle.sin(), inset + r - r * angle.cos());
    }
    accumulated += corner_arc;

    // 3. Right straight
    if dist < accumulated + straight_h {
        let frac = (dist - accumulated) / straight_h;
        return (width - inset, inset + r + frac * straight_h);
    }
    accumulated += straight_h;

    // 4. Bottom-right corner arc
    if dist < accumulated + corner_arc {
        let angle = ((dist - accumulated) / corner_arc) * (std::f32::consts::PI * 0.5);
        return (width - inset - r + r * angle.cos(), height - inset - r + r * angle.sin());
    }
    accumulated += corner_arc;

    // 5. Bottom straight
    if dist < accumulated + straight_w {
        let frac = (dist - accumulated) / straight_w;
        return (width - inset - r - frac * straight_w, height - inset);
    }
    accumulated += straight_w;

    // 6. Bottom-left corner arc
    if dist < accumulated + corner_arc {
        let angle = ((dist - accumulated) / corner_arc) * (std::f32::consts::PI * 0.5);
        return (inset + r - r * angle.sin(), height - inset - r + r * angle.cos());
    }
    accumulated += corner_arc;

    // 7. Left straight
    if dist < accumulated + straight_h {
        let frac = (dist - accumulated) / straight_h;
        return (inset, height - inset - r - frac * straight_h);
    }
    accumulated += straight_h;

    // 8. Top-left corner arc
    let angle = ((dist - accumulated) / corner_arc.max(0.001)) * (std::f32::consts::PI * 0.5);
    (inset + r - r * angle.cos(), inset + r - r * angle.sin())
}
