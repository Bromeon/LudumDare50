extends Spatial

var velTarget: float = 0.0;
var velocity: float = 0.0;

export var velScale = 10.0;

onready var propellers = [$"Propeller Left", $"Propeller Right"]

func setVelTarget(target):
	self.velTarget = target;

func _process(delta):
	velocity = lerp(velocity, velTarget, delta);
	for propeller in propellers:
		propeller.rotate_x(velocity * velScale * delta);
