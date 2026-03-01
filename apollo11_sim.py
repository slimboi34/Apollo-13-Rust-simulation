import numpy as np
import matplotlib.pyplot as plt
from scipy.integrate import solve_ivp
from matplotlib.animation import FuncAnimation

# Constants for Earth-Moon system
G = 6.67430e-11 # Gravitational constant (m^3 kg^-1 s^-2)
M_E = 5.972e24   # Mass of Earth (kg)
M_M = 7.348e22   # Mass of Moon (kg)
D_EM = 3.844e8   # Distance between Earth and Moon (m)

# Characteristic quantities for non-dimensionalization
L_star = D_EM
M_star = M_E + M_M
T_star = np.sqrt(L_star**3 / (G * M_star))

# Non-dimensional masses
mu = M_M / M_star
mu_E = 1 - mu

# Positions of Earth and Moon in the rotating frame
x_E = -mu
x_M = 1 - mu

def r3bp_eqs(t, state):
    """
    Equations of motion for the Circular Restricted Three-Body Problem (CR3BP).
    """
    x, y, z, vx, vy, vz = state
    
    r_E = np.sqrt((x - x_E)**2 + y**2 + z**2)
    r_M = np.sqrt((x - x_M)**2 + y**2 + z**2)
    
    # Gravitational forces + centrifugal + coriolis
    ax = 2*vy + x - mu_E * (x - x_E) / r_E**3 - mu * (x - x_M) / r_M**3
    ay = -2*vx + y - mu_E * y / r_E**3 - mu * y / r_M**3
    az = -mu_E * z / r_E**3 - mu * z / r_M**3
    
    return [vx, vy, vz, ax, ay, az]

# Free-Return Trajectory Approximation (Iterative guess based on Apollo 11 params)
# Starting near Earth, aiming to loop around the Moon and return
x0 = x_E + (6371e3 + 200e3) / L_star  # Start 200km above Earth
y0 = 0.0
z0 = 0.0
vx0 = 0.0
vy0 = 10.93e3 * T_star / L_star # roughly 10.9 km/s TLI burn
vz0 = 0.0

initial_state = [x0, y0, z0, vx0, vy0, vz0]

# Time span for the simulation (approx 8-10 days normalized)
t_span = (0, 7.0) 
t_eval = np.linspace(t_span[0], t_span[1], 1000)

print("Solving the Restricted Three-Body Problem...")
solution = solve_ivp(r3bp_eqs, t_span, initial_state, method='Radau', t_eval=t_eval, rtol=1e-8, atol=1e-8)
print("Simulation complete! Generating animation...")

# Extract results
x = solution.y[0]
y = solution.y[1]
z = solution.y[2]

# -- Customizing the visually breathtaking plot --
plt.style.use('dark_background')
fig = plt.figure(figsize=(10, 8))
ax = fig.add_subplot(111, projection='3d')
ax.set_facecolor('black')
fig.patch.set_facecolor('black')

# Plot Earth
u = np.linspace(0, 2 * np.pi, 100)
v = np.linspace(0, np.pi, 100)
R_E = 6371e3 / L_star
r_earth_x = x_E + R_E * np.outer(np.cos(u), np.sin(v))
r_earth_y = R_E * np.outer(np.sin(u), np.sin(v))
r_earth_z = R_E * np.outer(np.ones(np.size(u)), np.cos(v))
ax.plot_surface(r_earth_x, r_earth_y, r_earth_z, color='dodgerblue', alpha=0.8)

# Plot Moon
R_M = 1737e3 / L_star
r_moon_x = x_M + R_M * np.outer(np.cos(u), np.sin(v))
r_moon_y = R_M * np.outer(np.sin(u), np.sin(v))
r_moon_z = R_M * np.outer(np.ones(np.size(u)), np.cos(v))
ax.plot_surface(r_moon_x, r_moon_y, r_moon_z, color='lightgray', alpha=1.0)

# Path and spacecraft
path_line, = ax.plot([], [], [], color='cyan', lw=1.5, alpha=0.8, label='Apollo 11 Trajectory')
spacecraft, = ax.plot([], [], [], 'wo', markersize=6)

ax.set_xlim([-0.2, 1.2])
ax.set_ylim([-0.5, 0.5])
ax.set_zlim([-0.5, 0.5])
ax.set_title("Apollo 11 Free-Return Trajectory (Rotating Frame)", color='white', fontsize=14)
ax.axis('off') # Remove axis for a cinematic look

def init():
    path_line.set_data([], [])
    path_line.set_3d_properties([])
    spacecraft.set_data([], [])
    spacecraft.set_3d_properties([])
    return path_line, spacecraft

def animate(i):
    # Trail effect
    tail_len = min(i, 200)
    path_line.set_data(x[i-tail_len:i], y[i-tail_len:i])
    path_line.set_3d_properties(z[i-tail_len:i])
    
    spacecraft.set_data([x[i]], [y[i]])
    spacecraft.set_3d_properties([z[i]])
    return path_line, spacecraft

anim = FuncAnimation(fig, animate, init_func=init, frames=len(x), interval=20, blit=True)

plt.legend()
plt.tight_layout()
plt.show()

# To save: anim.save('apollo11_simulation.mp4', writer='ffmpeg', fps=30)
