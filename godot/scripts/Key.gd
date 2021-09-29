extends MeshInstance

const distance = Vector3(0, -1, 0)
var pressed = false

# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.

func press():
	if not pressed:
		self.translate(distance)
		pressed = true

func unpress():
	if pressed:
		self.translate(-distance)
		pressed = false
