% apollo11_simulation.m
% Apollo 11 Free-Return Trajectory Simulation in the Restricted 3-Body Problem
% A visually impactful portfolio piece

close all; clear; clc;

% -- 1. Constants and Normalization --
G = 6.67430e-11; % m^3 kg^-1 s^-2
M_E = 5.972e24;  % kg (Earth)
M_M = 7.348e22;  % kg (Moon)
D_EM = 3.844e8;  % m (Distance)

% Characterstic units
L_star = D_EM;
M_star = M_E + M_M;
T_star = sqrt(L_star^3 / (G * M_star));

mu = M_M / M_star;
mu_E = 1 - mu;

% Earth and Moon positions
x_E = -mu;
x_M = 1 - mu;

% -- 2. Initial Conditions (TLI Burn approximation) --
r_p_earth = (6371e3 + 200e3) / L_star; % 200km parking orbit
x0 = x_E + r_p_earth;
v_tli = 10.93e3 * T_star / L_star;

initial_state = [x0; 0; 0; 0; v_tli; 0];
tspan = [0 7.0]; % Normalized time

% -- 3. Solve ODE --
options = odeset('RelTol', 1e-8, 'AbsTol', 1e-8);
[t, Y] = ode45(@(t, y) r3bp(t, y, mu, mu_E, x_E, x_M), tspan, initial_state, options);

x = Y(:,1); y = Y(:,2); z = Y(:,3);

% -- 4. Breathtaking Visualization --
fig = figure('Name', 'Apollo 11 Simulation', 'Color', 'k', 'Position', [100 100 1000 700]);
ax = axes('Parent', fig, 'Color', 'k');
hold on;
axis equal;
grid off;
ax.XColor = 'none'; ax.YColor = 'none'; ax.ZColor = 'none';
view(3);

% Plot Earth
[X, Y_sphere, Z] = sphere(50);
R_E_norm = 6371e3 / L_star;
surf(X*R_E_norm + x_E, Y_sphere*R_E_norm, Z*R_E_norm, ...
    'FaceColor', [0.1, 0.4, 0.8], 'EdgeColor', 'none');

% Plot Moon
R_M_norm = 1737e3 / L_star;
surf(X*R_M_norm + x_M, Y_sphere*R_M_norm, Z*R_M_norm, ...
    'FaceColor', [0.8, 0.8, 0.8], 'EdgeColor', 'none');

% Setup animation objects
trajectory = plot3(NaN, NaN, NaN, 'c', 'LineWidth', 2);
spacecraft = plot3(NaN, NaN, NaN, 'wo', 'MarkerFaceColor', 'w', 'MarkerSize', 6);
title('Apollo 11 Free Return Trajectory', 'Color', 'w', 'FontSize', 16);

light('Position',[-1 0.5 0.5],'Style','local','Color',[1 1 1]);
lighting gouraud;
material dull;

xlim([-0.2, 1.2]); ylim([-0.5, 0.5]); zlim([-0.2, 0.2]);

% Animate
trail_len = 100;
for i = 1:5:length(t)
    start_idx = max(1, i - trail_len);
    set(trajectory, 'XData', x(start_idx:i), 'YData', y(start_idx:i), 'ZData', z(start_idx:i));
    set(spacecraft, 'XData', x(i), 'YData', y(i), 'ZData', z(i));
    drawnow;
    pause(0.01);
end

% Keep final trajectory plotted
set(trajectory, 'XData', x, 'YData', y, 'ZData', z, 'Color', [0 1 1 0.5]);

% -- Helper Function --
function dydt = r3bp(~, y, mu, mu_E, x_E, x_M)
    r_E = sqrt((y(1)-x_E)^2 + y(2)^2 + y(3)^2);
    r_M = sqrt((y(1)-x_M)^2 + y(2)^2 + y(3)^2);
    
    dydt = zeros(6,1);
    dydt(1:3) = y(4:6);
    dydt(4) = 2*y(5) + y(1) - mu_E*(y(1)-x_E)/r_E^3 - mu*(y(1)-x_M)/r_M^3;
    dydt(5) = -2*y(4) + y(2) - mu_E*y(2)/r_E^3 - mu*y(2)/r_M^3;
    dydt(6) = -mu_E*y(3)/r_E^3 - mu*y(3)/r_M^3;
end
