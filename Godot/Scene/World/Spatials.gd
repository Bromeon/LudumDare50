extends Spatial


const RAY_LENGTH = 1000.0


var matDefault: SpatialMaterial
var matHighlighted: SpatialMaterial

var lastHighlightedObj: Spatial


# Called when the node enters the scene tree for the first time.
func _ready():
	var scene = preload("res://Scene/Objects/Structure.tscn")
	$SpatialObjects.load(scene)

	matDefault = SpatialMaterial.new()
	matHighlighted = SpatialMaterial.new()
	matHighlighted.albedo_color = Color.crimson


func _process(_delta):
	raycast()


func updateHovered() -> void:
	pass


func raycast():
	var localMousePos = get_viewport().get_mouse_position()

	var spaceRid = get_world().space
	var spaceState = PhysicsServer.space_get_direct_state(spaceRid)

	var origin = $Camera.project_ray_origin(localMousePos)
	var normal = $Camera.project_ray_normal(localMousePos)

	var result = spaceState.intersect_ray(origin, origin + normal * RAY_LENGTH)
	#print(result)

	var collider = result.get("collider")
	if collider is StaticBody:
		var parent: Spatial = collider.get_parent()
		parent.applyMaterial(matHighlighted)

		lastHighlightedObj = parent

	elif lastHighlightedObj != null:
		lastHighlightedObj.applyMaterial(matDefault)


# Mouse position projected onto XY plane (z=0)
func projectMousePos(localMousePos: Vector2) -> Vector3:
	#var mousePos = get_viewport().get_mouse_position()
	var origin = $Camera.project_ray_origin(localMousePos)
	var normal = $Camera.project_ray_normal(localMousePos)
	var z = 0

	#var spaceState = get_world().direct_space_state
	var projection = Plane(Vector3.BACK, z).intersects_ray(origin, normal)

	# Note: due to rounding errors, z coordinate of projection can be -0.00004, so floor() makes it -1
	# Thus, manually set it to desired coordinate
	projection.z = z

	#print("Projected: ", projection)
	return projection
