import pyplotters as plt
import ndarray as np

# Init function is run once
def init():
	global x, y, prev
	k = 100.0
	x = np.arange(-k, k) / k
	y = np.zeros(len(x))
	prev = y
	y[len(y)//3] = 1.0
	
	
if 'prev' not in globals():
	init()
	
# Boundary conditions
y[0] = 0.0
y[-1] = 0.0

# Copy previous state
tmp = y.__copy__()

# Calculate next time step
deriv = (y[:-2] + y[1:-1] * -2.0 + y[2:])
y[1:-1] = y[1:-1] * 2.0 - prev[1:-1] + deriv / 2.0

# Assign previous state
prev = tmp

# Graphing
plt.title("Wave equation")
plt.xlim(x[0], x[-1])
plt.ylim(-1.0, 1.0)
plt.legend()

plt.plot(x, y, label="Waves")
