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

// New uniforms for direction, width, glow, and style
uniform int u_tracer_direction; // 0 = Clockwise, 1 = CounterClockwise
uniform float u_tracer_width;   // Moving line width (e.g. 1.0 - 24.0)
uniform float u_tracer_glow;    // Line glow / bloom intensity (0.0 - 1.0)
uniform int u_tracer_style;     // 0 = AuroraWave, 1 = NeonGlow, 2 = ElectricPulse

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

// Pseudo-random hash
float hash(float n) {
    return fract(sin(n) * 43758.5453123);
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

    // Moving Tracer Line (Orbit Beam with custom Direction, Width, Glow & Styles)
    if (u_tracer_enabled == 1 && (u_mode == 0 || u_mode == 1)) {
        // Direction handling: Clockwise (0) vs Counter-Clockwise (1)
        float dt = (u_tracer_direction == 1) 
            ? fract(t_param - u_tracer_position)
            : fract(u_tracer_position - t_param);

        float tail_len = max(u_tracer_tail, 0.05);

        if (dt <= tail_len) {
            float tail_progress = dt / tail_len; // 0.0 at head, 1.0 at tail end
            float tail_falloff = pow(1.0 - tail_progress, 1.9);

            // Baseline half-width with audio transient pulse
            float base_half_w = max(u_tracer_width * 0.5, 0.75);
            float active_half_w = base_half_w * (1.0 + u_beat_energy * 0.45);
            float active_glow = max(u_tracer_glow, 0.05) * (1.0 + u_beat_energy * 0.6);

            float tracer_brightness = 0.0;
            vec3 tracer_color = vec3(1.0);

            if (u_tracer_style == 0) {
                // Style 0: Aurora Wave (Instagram Reel fluid organic wave texture)
                // Multi-frequency sinusoidal turbulence displacing the line contour
                float t_scaled = t_param * 40.0;
                float wave1 = sin(t_scaled + u_time * 4.0) * (active_half_w * 0.6);
                float wave2 = sin(t_scaled * 2.2 - u_time * 6.5) * (active_half_w * 0.35);
                float wave3 = cos(t_param * 20.0 + u_time * 2.5) * (active_half_w * 0.5);
                float total_wave = (wave1 + wave2 + wave3) * (1.0 - tail_progress * 0.6);

                float wave_dist = abs(dist_from_edge - total_wave);

                // High-luminance crystalline filament core
                float core = smoothstep(active_half_w, 0.4, wave_dist) * 1.5;

                // Flowing multi-spectral aurora ribbons (cyan, emerald, violet, base)
                float aurora_phase = sin(t_param * 16.0 + u_time * 3.0) * 0.5 + 0.5;
                vec3 aurora_cyan = vec3(0.0, 0.98, 0.85);
                vec3 aurora_emerald = vec3(0.1, 0.96, 0.42);
                vec3 aurora_violet = vec3(0.72, 0.32, 0.98);
                
                vec3 ribbon_tint = mix(aurora_cyan, aurora_emerald, aurora_phase);
                ribbon_tint = mix(ribbon_tint, aurora_violet, sin(u_time * 1.8 + t_param * 8.0) * 0.5 + 0.5);
                ribbon_tint = mix(ribbon_tint, base_color, 0.35);

                // Volumetric aurora plume bloom
                float bloom_spread = active_half_w * 2.4 + 4.0;
                float shimmer = 0.88 + 0.12 * sin(t_param * 90.0 + u_time * 15.0);
                float plume = exp(-wave_dist / bloom_spread) * active_glow * 1.6 * shimmer;

                tracer_brightness = (core + plume) * tail_falloff;
                tracer_color = mix(ribbon_tint, vec3(1.0), core * 0.7);

            } else if (u_tracer_style == 1) {
                // Style 1: Neon Glow (Razor-sharp core + deep soft bloom)
                float core = smoothstep(active_half_w, 0.3, dist_from_edge) * 1.8;
                float bloom = exp(-dist_from_edge / (active_half_w * 2.8 + 6.0)) * active_glow * 1.7;

                tracer_brightness = (core + bloom) * tail_falloff;
                vec3 neon_tint = mix(u_primary_color, vec3(0.3, 0.8, 1.0), 0.5);
                tracer_color = mix(neon_tint, vec3(1.0), core * 0.8);

            } else {
                // Style 2: Electric Pulse (Energetic plasma micro-arcs)
                float noise_val = hash(floor(t_param * 90.0 + u_time * 24.0));
                float arc_offset = (noise_val - 0.5) * (active_half_w * 0.9);
                float arc_dist = abs(dist_from_edge - arc_offset);

                float core = smoothstep(active_half_w * 0.7, 0.2, arc_dist) * 1.6;
                float spark = (noise_val > 0.7) ? 1.4 : 0.8;
                float bloom = exp(-arc_dist / (active_half_w * 1.6 + 3.0)) * active_glow * 1.3;

                tracer_brightness = (core * spark + bloom) * tail_falloff;
                vec3 electric_tint = vec3(0.4, 0.7, 1.0);
                tracer_color = mix(electric_tint, vec3(1.0), core * 0.85);
            }

            final_rgb += tracer_color * tracer_brightness;
            final_alpha = max(final_alpha, clamp(tracer_brightness * 0.95, 0.0, 1.0));
        }
    }

    // Output transparent click-through composite
    frag_color = vec4(final_rgb, clamp(final_alpha, 0.0, 1.0));
}
