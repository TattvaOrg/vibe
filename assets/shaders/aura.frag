#version 330 core

in vec2 v_uv;
out vec4 frag_color;

uniform vec2 u_resolution;
uniform float u_time;
uniform float u_thickness;
uniform float u_diffusion;
uniform float u_intensity;
uniform vec3 u_primary_color;
uniform vec3 u_secondary_color;
uniform float u_color_weight;
uniform float u_corner_radius;
uniform float u_screen_inset;
uniform int u_tracer_enabled;
uniform float u_tracer_position;
uniform float u_tracer_tail;
uniform float u_beat_energy;
uniform int u_mode;

#define PI 3.14159265359
#define TWO_PI 6.28318530718

// Signed distance to a rounded box / squircle
float rounded_box_sdf(vec2 p, vec2 b, float r) {
    vec2 q = abs(p) - b + vec2(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r;
}

// Convert angle [-PI, PI] to normalized perimeter coordinate [0, 1]
float angle_to_t(vec2 p) {
    float angle = atan(p.y, p.x);
    if (angle < 0.0) angle += TWO_PI;
    return angle / TWO_PI;
}

void main() {
    vec2 pixel_pos = v_uv * u_resolution;
    vec2 half_res = u_resolution * 0.5;
    vec2 centered = pixel_pos - half_res;

    // Define outer squircle boundary with inset
    vec2 box_extents = half_res - vec2(u_screen_inset);
    float corner_r = min(u_corner_radius, min(box_extents.x, box_extents.y));

    // Distance from the display perimeter border
    // Negative inside the screen, positive outside
    float d = rounded_box_sdf(centered, box_extents, corner_r);
    float dist_from_edge = abs(d);

    // Dynamic beat expansion
    float beat_boost = u_beat_energy * 0.35;
    float current_thickness = u_thickness * (1.0 + beat_boost);
    float current_intensity = u_intensity * (1.0 + beat_boost * 0.5);

    // Compute perimeter angular coordinate for color gradients & wave
    float t_param = angle_to_t(centered);

    // Mode-specific color motion
    vec3 base_color;
    if (u_mode == 2) {
        // Fluid Wave: color ribbon flows around perimeter
        float wave = sin((t_param + u_time * 0.15) * TWO_PI) * 0.5 + 0.5;
        base_color = mix(u_primary_color, u_secondary_color, wave);
    } else if (u_mode == 3) {
        // Ambient Breathe: gentle tempo-synced breathing luminescence
        float breathe = sin(u_time * 1.5) * 0.5 + 0.5;
        current_intensity *= (0.7 + 0.3 * breathe);
        base_color = mix(u_primary_color, u_secondary_color, u_color_weight);
    } else {
        // Standard primary/secondary gradient blend along display
        float grad = smoothstep(0.0, 1.0, (sin(t_param * TWO_PI) * 0.5 + 0.5));
        base_color = mix(u_primary_color, u_secondary_color, mix(grad, u_color_weight, 0.4));
    }

    // Volumetric Ambient Aura (Gaussian falloff inward from the edge)
    float diffusion_factor = max(u_diffusion * current_thickness * 1.8, 1.0);
    float aura_alpha = exp(-dist_from_edge / diffusion_factor) * current_intensity;
    
    // Mask out outside screen boundary
    if (d > 0.0) {
        aura_alpha *= exp(-d * 0.5);
    }

    vec3 final_rgb = base_color * aura_alpha;
    float final_alpha = aura_alpha;

    // Signature White Perimeter Tracer Beam (Orbit Line)
    if (u_tracer_enabled == 1 && (u_mode == 0 || u_mode == 1)) {
        // Find distance along perimeter from current tracer position
        float dt = fract(u_tracer_position - t_param);
        float tail_len = max(u_tracer_tail, 0.05);

        if (dt <= tail_len) {
            // Comet tail falloff from head (dt=0) to tail end (dt=tail_len)
            float tail_falloff = pow(1.0 - (dt / tail_len), 2.2);

            // Razor-sharp 2px pure white filament core
            float core_dist = dist_from_edge;
            float core_beam = smoothstep(2.5, 0.5, core_dist);

            // Soft 8px outer bloom around the white tracer path
            float bloom_beam = exp(-core_dist / 6.0) * 0.6;

            float tracer_brightness = (core_beam * 1.2 + bloom_beam) * tail_falloff;
            
            // On beat kick, flash the tracer head with HDR intensity
            tracer_brightness *= (1.0 + u_beat_energy * 0.6);

            vec3 tracer_color = vec3(1.0, 1.0, 1.0);
            final_rgb += tracer_color * tracer_brightness;
            final_alpha = max(final_alpha, tracer_brightness * 0.95);
        }
    }

    // Output transparent click-through composite
    frag_color = vec4(final_rgb, clamp(final_alpha, 0.0, 1.0));
}
