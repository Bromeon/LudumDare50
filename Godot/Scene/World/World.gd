extends Spatial

const SpatialApi = preload("res://Native/SpatialApi.gdns")
const AddStructure = preload("res://Native/AddStructure.gdns")
const RisingText = preload("res://Scene/Util/RisingText.tscn")


const RAY_LENGTH = 1000.0

const BUILD_RADIUS = 6.0


var matDefault: SpatialMaterial
var matHighlighted: SpatialMaterial
var matSelected: SpatialMaterial
var matAffected: SpatialMaterial
var matPowered: SpatialMaterial

var lastHoveredObj: Spatial = null
var selectedObj: Spatial = null

onready var ghostStc: Spatial = $Ghosts/Pump
onready var ghostPipe: Spatial = $Ghosts/Pipe


# ----------------------------------------------------------------------------------------------------------------------------------------------
# APIs called from Rust

func alignPipe(pipe: Spatial, from: Vector3, to: Vector3) -> void:
	var structureWidth = 0.2
	var dist = from.distance_to(to) - structureWidth

	pipe.transform = Transform() \
		.translated(from) \
		.scaled(Vector3(1, 1, 0.5 * dist)) \
		.translated(-from)

	pipe.transform.origin = from
	pipe.look_at(to, Vector3.UP)


func setPowered(instanceId: int, powered: bool) -> void:
	#print("setPowered: ", instanceId, " powered=", powered)
	var obj: Spatial = instance_from_id(instanceId)

	if powered:
		obj.applyMaterial(matPowered)
	else:
		obj.applyMaterial(matDefault)


# ----------------------------------------------------------------------------------------------------------------------------------------------

# Called when the node enters the scene tree for the first time.
func _ready():
	var scenes = {
		Water = preload("res://Scene/Objects/Water.tscn"),
		Ore = preload("res://Scene/Objects/Ore.tscn"),
		Pump = preload("res://Scene/Objects/Pump.tscn"),
		Irrigation = preload("res://Scene/Objects/Irrigation.tscn"),
		Pipe = preload("res://Scene/Objects/Pipe.tscn"),
	}

	$SpatialApi.load(scenes)

	matDefault = SpatialMaterial.new()
	matHighlighted = SpatialMaterial.new()
	matHighlighted.albedo_color = Color.crimson
	matSelected = SpatialMaterial.new()
	matSelected.albedo_color = Color.darksalmon
	matAffected = SpatialMaterial.new()
	matAffected.albedo_color = Color.goldenrod
	matPowered = SpatialMaterial.new()
	matPowered.albedo_color = Color.blue

	$BuildRadius.scale = Vector3(BUILD_RADIUS, 1.01, BUILD_RADIUS)


func _process(dt: float):
	# Escape
	if Input.is_action_just_pressed("ui_cancel"):
		get_tree().quit()
		return

	if Input.is_action_just_pressed("ui_restart"):
		restartGame()
		return

	$SpatialApi.update_frame_count()
	var result = $SpatialApi.update_blight(dt)
	for id in result.removed_pipe_ids:
		var node = instance_from_id(id)
		node.queue_free()

	var amounts = $SpatialApi.update_amounts()
	if amounts != null:
		$HUD.set_ore_amount(amounts.total_ore)

		var remain = amounts.ore_fields_remaining_amounts
		print("Remain: ", remain)
		for id in remain:
			var oreAmount = remain[id]
			var oreObj = instance_from_id(id)

			oreObj.updateAmount(oreAmount)

		for i in amounts.animated_positions.size():
			var amount = amounts.animated_diffs[i]
			var pos = amounts.animated_positions[i]
			var pos3d = Vector3(pos.x, 2, pos.y)

			var rt = RisingText.instance()
			rt.init(amount)
			rt.translate(pos3d)

			$SceneUi.add_child(rt)




	handleMouseInteraction()


func handleMouseInteraction():
	# Invalidate deleted
	if lastHoveredObj != null && !is_instance_valid(lastHoveredObj):
		lastHoveredObj = null
	if selectedObj != null && !is_instance_valid(selectedObj):
		selectedObj = null

	# Reset default materials
	for node in $SpatialApi/Structures.get_children():
		if node != selectedObj:
			node.applyMaterial(matDefault)

	# See if mouse hits something
	var localMousePos = get_viewport().get_mouse_position()
	var collider = raycastMouseObject(localMousePos)

	if collider is StaticBody:
		var hovered: Spatial = collider.get_parent()

		# Left click selected
		if Input.is_action_just_pressed("left_click"):
			updateSelected(hovered)

		# Just hovering (or clicked + hovered)
		updateHovered(hovered)

		# Hide ghosts
		hideGhosts()

	# Hovering outside	
	else:
		$EffectRadius.visible = false

		var groundPosInRange = null
		if selectedObj != null:
			var groundPos = raycastMouseGround(localMousePos)
			if groundPos.distance_squared_to(selectedObj.translation) < BUILD_RADIUS * BUILD_RADIUS:
				groundPosInRange = groundPos

		# De-selected (click on ground)
		if Input.is_action_just_pressed("left_click"):
			$BuildRadius.visible = false
			selectedObj = null
			return

		# Place building
		if Input.is_action_just_pressed("right_click"):
			if groundPosInRange != null:
				var add = AddStructure.new()
				add.position = groundPosInRange
				add.structure_ty = "Pump"
				add.pipe_from_obj = selectedObj

				var id = $SpatialApi.add_structure(add)
				updateSelected(instance_from_id(id))
			return

		# Drag ghost
		if groundPosInRange != null:
			showGhosts(selectedObj.translation, groundPosInRange)

		else:
			hideGhosts()


func updateSelected(obj) -> void:
	obj.applyMaterial(matSelected)
	selectedObj = obj

	$BuildRadius.translation = obj.translation
	$BuildRadius.visible = true

	
func updateHovered(obj) -> void:
	if obj != selectedObj:
		obj.applyMaterial(matHighlighted)
	lastHoveredObj = obj

	# Mark affected buildings (in effect radius)
	var queried = $SpatialApi.query_effect_radius(obj)
	if queried == null:
		printerr("SHOULD NOT HAPPEN")
		print("obj: ", obj)

	$EffectRadius.translation = obj.translation
	$EffectRadius.visible = true
	$EffectRadius.scale = Vector3(queried.radius, 1.01, queried.radius)

	for id in queried.affected_ids:
		var node = instance_from_id(id)
		if node != obj && node != selectedObj:
			node.applyMaterial(matAffected)


func hideGhosts() -> void:
	ghostStc.visible = false
	ghostPipe.visible = false


# from: selected pos
# to:  pos of new building
func showGhosts(from: Vector3, to: Vector3) -> void:
	ghostStc.visible = true
	ghostStc.translation = to

	ghostPipe.visible = true
	alignPipe(ghostPipe, from, to)


# Returns object hit by mouse, or null if on ground
func raycastMouseObject(localMousePos: Vector2):
	var spaceRid = get_world().space
	var spaceState = PhysicsServer.space_get_direct_state(spaceRid)

	var camera: Camera = get_node("/root/World/Camera")
	var origin = camera.project_ray_origin(localMousePos)
	var normal = camera.project_ray_normal(localMousePos)

	var result = spaceState.intersect_ray(origin, origin + normal * RAY_LENGTH)
	return result.get("collider")


# Mouse position projected onto XY plane (z=0)
func raycastMouseGround(localMousePos: Vector2) -> Vector3:
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


func restartGame():
	var success = get_tree().reload_current_scene()
	if success != OK:
		printerr("Error reloading game scene")
