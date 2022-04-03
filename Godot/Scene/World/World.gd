extends Spatial

const SpatialApi = preload("res://Native/SpatialApi.gdns")


const RAY_LENGTH = 1000.0

const EFFECT_RADIUS = 4.0


var matDefault: SpatialMaterial
var matHighlighted: SpatialMaterial
var matAffected: SpatialMaterial

var lastHighlightedObj: Spatial = null


# Called when the node enters the scene tree for the first time.
func _ready():
	var scenes = {
		Water = preload("res://Scene/Objects/Water.tscn"),
		Ore = preload("res://Scene/Objects/Ore.tscn"),
		Pump = preload("res://Scene/Objects/Pump.tscn"),
		Irrigation = preload("res://Scene/Objects/Irrigation.tscn"),
	}

	$SpatialApi.load(scenes)

	matDefault = SpatialMaterial.new()
	matHighlighted = SpatialMaterial.new()
	matHighlighted.albedo_color = Color.crimson
	matAffected = SpatialMaterial.new()
	matAffected.albedo_color = Color.goldenrod

	$EffectRadius.scale = 2 * Vector3(EFFECT_RADIUS, 1, EFFECT_RADIUS)


func _process(dt: float):
	# Escape
	if Input.is_action_just_pressed("ui_cancel"):
		get_tree().quit()
		return

	$SpatialApi.update_blight_impact(dt)

	raycast()


func updateHovered() -> void:
	pass


func raycast():
	var localMousePos = get_viewport().get_mouse_position()

	var spaceRid = get_world().space
	var spaceState = PhysicsServer.space_get_direct_state(spaceRid)

	var camera: Camera = get_node("/root/World/Camera")
	var origin = camera.project_ray_origin(localMousePos)
	var normal = camera.project_ray_normal(localMousePos)

	var result = spaceState.intersect_ray(origin, origin + normal * RAY_LENGTH)
	#print(result)

	for node in $SpatialApi/Structures.get_children():
		node.applyMaterial(matDefault)

	var collider = result.get("collider")
	if collider is StaticBody:
		var hovered: Spatial = collider.get_parent()

		lastHighlightedObj = hovered
		hovered.applyMaterial(matHighlighted)

		$EffectRadius.translation = lastHighlightedObj.translation
		$EffectRadius.visible = true

		var affectedIds = $SpatialApi.query_radius(hovered.global_transform.origin, EFFECT_RADIUS)
		for id in affectedIds:
			var node = instance_from_id(id)
			if node != hovered:
				node.applyMaterial(matAffected)



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
