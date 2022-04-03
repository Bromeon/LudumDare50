extends Spatial

const SpatialApi = preload("res://Native/SpatialApi.gdns")


const RAY_LENGTH = 1000.0

const EFFECT_RADIUS = 4.0
const BUILD_RADIUS = 6.0


var matDefault: SpatialMaterial
var matHighlighted: SpatialMaterial
var matSelected: SpatialMaterial
var matAffected: SpatialMaterial

var lastHighlightedObj: Spatial = null
var selectedObj: Spatial = null

var ghost: Spatial


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
	matSelected = SpatialMaterial.new()
	matSelected.albedo_color = Color.darksalmon
	matAffected = SpatialMaterial.new()
	matAffected.albedo_color = Color.goldenrod

	$EffectRadius.scale = 2 * Vector3(EFFECT_RADIUS, 1, EFFECT_RADIUS)
	$BuildRadius.scale = 2 * Vector3(BUILD_RADIUS, 1.01, BUILD_RADIUS)

	ghost = $Ghosts/Pump


func _process(dt: float):
	# Escape
	if Input.is_action_just_pressed("ui_cancel"):
		get_tree().quit()
		return

	$SpatialApi.update_blight_impact(dt)
	$HUD.set_ore_amount($SpatialApi.get_ore_amount())

	raycast()


func updateHovered() -> void:
	pass


func raycast():
	# Invalidate deleted
	if lastHighlightedObj != null && !is_instance_valid(lastHighlightedObj):
		lastHighlightedObj = null
	if selectedObj != null && !is_instance_valid(selectedObj):
		selectedObj = null

	var localMousePos = get_viewport().get_mouse_position()

	var spaceRid = get_world().space
	var spaceState = PhysicsServer.space_get_direct_state(spaceRid)

	var camera: Camera = get_node("/root/World/Camera")
	var origin = camera.project_ray_origin(localMousePos)
	var normal = camera.project_ray_normal(localMousePos)

	var result = spaceState.intersect_ray(origin, origin + normal * RAY_LENGTH)
	#print(result)

	for node in $SpatialApi/Structures.get_children():
		if node != selectedObj:
			node.applyMaterial(matDefault)

	var collider = result.get("collider")
	if collider is StaticBody:
		var hovered: Spatial = collider.get_parent()

		# Left click selected
		if Input.is_action_just_pressed("left_click"):
			hovered.applyMaterial(matSelected)
			selectedObj = hovered
			$BuildRadius.translation = lastHighlightedObj.translation
			$BuildRadius.visible = true

		# Just hovering
		lastHighlightedObj = hovered
		if hovered != selectedObj:
			hovered.applyMaterial(matHighlighted)

		$EffectRadius.translation = lastHighlightedObj.translation
		$EffectRadius.visible = true

		# Mark affected buildings (in effect radius)
		var affectedIds = $SpatialApi.query_radius(hovered.global_transform.origin, EFFECT_RADIUS)
		for id in affectedIds:
			var node = instance_from_id(id)
			if node != hovered && node != selectedObj:
				node.applyMaterial(matAffected)

		# Hide ghost
		ghost.visible = false

	# Hovering outside	
	else:
		$EffectRadius.visible = false

		var inBuildRadius: bool = false
		var groundPos = null
		if selectedObj:
			groundPos = projectMousePos(localMousePos)
			inBuildRadius = groundPos.distance_squared_to(selectedObj.translation) < BUILD_RADIUS * BUILD_RADIUS

		# De-selected (click on ground)
		if Input.is_action_just_pressed("left_click"):
			$BuildRadius.visible = false
			selectedObj = null

		# Place building
		elif Input.is_action_just_pressed("right_click"):
			if inBuildRadius:
				var id = $SpatialApi.add_structure(groundPos, "Pump")
				selectedObj = instance_from_id(id)

		# Drag ghost
		if inBuildRadius:
			ghost.visible = true
			ghost.translation = groundPos
		else:
			ghost.visible = false

			
# Mouse position projected onto XY plane (z=0)
func projectMousePos(localMousePos: Vector2) -> Vector3:
	#var mousePos = get_viewport().get_mouse_position()
	var origin = $Camera.project_ray_origin(localMousePos)
	var normal = $Camera.project_ray_normal(localMousePos)
	var y = 0

	#var spaceState = get_world().direct_space_state
	var projection = Plane(Vector3.UP, y).intersects_ray(origin, normal)

	# Note: due to rounding errors, z coordinate of projection can be -0.00004, so floor() makes it -1
	# Thus, manually set it to desired coordinate
	projection.y = y

	#print("Projected: ", projection)
	return projection
