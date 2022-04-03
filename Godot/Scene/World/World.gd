extends Spatial

const SpatialApi = preload("res://Native/SpatialApi.gdns")


const RAY_LENGTH = 1000.0

const EFFECT_RADIUS = 4.0


var matDefault: SpatialMaterial
var matHighlighted: SpatialMaterial
var matAffected: SpatialMaterial

var lastHighlightedObj: Spatial = null
var lastAffectedIds: Array = []


# Called when the node enters the scene tree for the first time.
func _ready():
	var scene = preload("res://Scene/Objects/Structure.tscn")
	$SpatialApi.load(scene)

	matDefault = SpatialMaterial.new()
	matHighlighted = SpatialMaterial.new()
	matHighlighted.albedo_color = Color.crimson
	matAffected = SpatialMaterial.new()
	matAffected.albedo_color = Color.goldenrod

	$EffectRadius.scale = Vector3(EFFECT_RADIUS, 1, EFFECT_RADIUS)


func _process(_delta):
    # Escape
    if Input.is_action_just_pressed("ui_cancel"):
        get_tree().quit()
        return

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

	var collider = result.get("collider")
	if collider is StaticBody:
		var hovered: Spatial = collider.get_parent()
		hovered.applyMaterial(matHighlighted)

		lastHighlightedObj = hovered
		$EffectRadius.translation = lastHighlightedObj.translation
		$EffectRadius.visible = true

		var affected = $SpatialApi.query_radius(hovered.global_transform.origin, EFFECT_RADIUS)
		updateAffected(hovered, affected)

	elif lastHighlightedObj != null:
		$EffectRadius.visible = false
		lastHighlightedObj.applyMaterial(matDefault)
		lastHighlightedObj = null
		updateAffected(null, [])


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


func updateAffected(affector, affectedIds: Array):
	for id in lastAffectedIds:
		var node = instance_from_id(id)
		if affector == null || node != affector:
			node.applyMaterial(matDefault)

	for id in affectedIds:
		var node = instance_from_id(id)
		if affector == null || node != affector:
			node.applyMaterial(matAffected)

	lastAffectedIds = affectedIds
