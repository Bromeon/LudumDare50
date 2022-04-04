extends Spatial

const SpatialApi = preload("res://Native/SpatialApi.gdns")
const AddStructure = preload("res://Native/AddStructure.gdns")
const RisingText = preload("res://Scene/Util/RisingText.tscn")


const RAY_LENGTH = 1000.0

const BUILD_RADIUS = 6.0

# FIXME duplicated in structure.rs
const IRRIGATION_COST = 50
const PUMP_COST = 15

var matDefault: SpatialMaterial
var matHighlighted: SpatialMaterial
var matSelected: SpatialMaterial
var matAffected: SpatialMaterial
var matPowered: SpatialMaterial

var lastHoveredObj: Spatial = null
var selectedObj: Spatial = null
var hasBuilt: bool = false

onready var ghostsStc: Array = [$SceneUi/Ghosts/Pump, $SceneUi/Ghosts/Irrigation]
onready var ghostPipe: Spatial = $SceneUi/Ghosts/Pipe


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

	obj.setPowered(powered);
	#if powered:
		#obj.applyMaterial(matPowered)
	#else:
		#obj.applyMaterial(matDefault)


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

	$SceneUi/BuildRadius.scale = Vector3(BUILD_RADIUS, 1.01, BUILD_RADIUS)


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

		var remain = amounts.remaining_resource_amounts
		for id in remain:
			var oreAmount = remain[id]
			var oreObj = instance_from_id(id)

			oreObj.updateAmount(oreAmount)

		for i in amounts.animated_positions.size():
			var amount = amounts.animated_diffs[i]
			var pos = amounts.animated_positions[i]
			var string = amounts.animated_strings[i]
			var pos3d = Vector3(pos.x, .2, pos.y)

			var rt = RisingText.instance()
			rt.init(amount, string)
			$SceneUi.add_child(rt)
			rt.translation = pos3d


	handleMouseInteraction()

var placedStructureType = "Pump"

func _input(event):
	if event is InputEventMouseButton:
		if event.is_pressed():
			if event.button_index == BUTTON_WHEEL_UP or event.button_index == BUTTON_WHEEL_DOWN:
				placedStructureType = "Pump" if placedStructureType == "Irrigation" else "Irrigation"
				print(placedStructureType)
		   

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

	$SceneUi/Control.rect_position = localMousePos + Vector2(30, -10)

	if collider is StaticBody:
		var hovered: Spatial = collider.get_parent()
		if $SpatialApi.can_build_from(hovered.get_instance_id()):
			updateTooltip(hovered, "Left click: select")
			
			# Left click selected
			if Input.is_action_just_pressed("left_click"):
				updateSelected(hovered)

		# Just hovering (or clicked + hovered)
		updateHovered(hovered)

		# Hide ghosts
		hideGhosts()

	# Hovering outside	
	else:
		$SceneUi/EffectRadius.visible = false


		var buildingCost = PUMP_COST if placedStructureType == "Pump" else IRRIGATION_COST
		var groundPosInRange = null
		if selectedObj != null:
			var s = str(placedStructureType, " (cost ", buildingCost, ")")
			updateTooltip(null, str("Right click: place ", s, "\nScroll wheel: switch building"))

			var groundPos = raycastMouseGround(localMousePos)
			if groundPos.distance_squared_to(selectedObj.translation) < BUILD_RADIUS * BUILD_RADIUS:
				groundPosInRange = groundPos

		else:
			if hasBuilt:
				updateTooltip(null, "Left click: select water or building")
			else:
				updateTooltip(null, "Left click: select water source")


		# De-selected (click on ground)
		if Input.is_action_just_pressed("left_click"):
			$SceneUi/BuildRadius.visible = false
			selectedObj = null
			return

		# Place building
		if Input.is_action_just_pressed("right_click"):
			if $SpatialApi.can_consume_ore(buildingCost):
				$SpatialApi.consume_ore(buildingCost)
				if groundPosInRange != null:
					var add = AddStructure.new()
					add.position = groundPosInRange
					add.structure_ty = placedStructureType
					add.pipe_from_obj = selectedObj
					
					var id = $SpatialApi.add_structure(add)
					Sfx.startMachineSound(groundPosInRange, id)
					updateSelected(instance_from_id(id))
					
					Sfx.placeItem()
					hasBuilt = true
			else:
				Sfx.wrong()
			return

		# Drag ghost
		if groundPosInRange != null:
			showGhosts(selectedObj.translation, groundPosInRange)

		else:
			hideGhosts()


func updateSelected(obj) -> void:
	obj.applyMaterial(matSelected)
	selectedObj = obj

	$SceneUi/BuildRadius.translation = obj.translation
	$SceneUi/BuildRadius.visible = true

	
func updateHovered(obj) -> void:
	if obj != selectedObj:
		obj.applyMaterial(matHighlighted)
	lastHoveredObj = obj

	# Mark affected buildings (in effect radius)
	var queried = $SpatialApi.query_effect_radius(obj)
	if queried == null:
		printerr("SHOULD NOT HAPPEN")
		print("obj: ", obj)

	$SceneUi/EffectRadius.translation = obj.translation
	$SceneUi/EffectRadius.visible = true
	$SceneUi/EffectRadius.scale = Vector3(queried.radius, 1.01, queried.radius)

	for id in queried.affected_ids:
		var node = instance_from_id(id)
		if node != obj && node != selectedObj:
			node.applyMaterial(matAffected)


func hideGhosts() -> void:
	for ghost in ghostsStc:
		ghost.visible = false
	ghostPipe.visible = false


# from: selected pos
# to:  pos of new building
func showGhosts(from: Vector3, to: Vector3) -> void:
	var ghostIdx = 0 if placedStructureType == "Pump" else 1
	for i in range(0,2):
		ghostsStc[i].visible = i == ghostIdx
	ghostsStc[ghostIdx].translation = to

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


# func hideTooltip() -> void:
# 	$SceneUi/Control/Label.text = ""

func updateTooltip(obj, tip = null) -> void:
	var label = $SceneUi/Control/Label
	if obj == null:
		label.text = tip

	elif tip == null:	
		label.text = $SpatialApi.get_structure_info(obj.get_instance_id(), false)
	
	else:
		var minimalStr = $SpatialApi.get_structure_info(obj.get_instance_id(), true)
		label.text = str(minimalStr, "\n", tip)
	
