extends Node

onready var machineSound = preload("res://SFX/machine_noise.ogg")

var sounds = {}

func startMachineSound(pos: Vector3, id: int):
	var player = AudioStreamPlayer3D.new()
	player.stream = machineSound
	player.transform.origin = pos
	player.play()
	player.unit_db = 20.0
	sounds[id] = player
	add_child(player)
	
func stopMachineSound(id: int):
	if sounds.has(id):
		sounds[id].stop()
		remove_child(sounds[id])
		sounds.erase(id)
	print(sounds)
	
func placeItem():
	$PlaceItem.play()

func wrong():
	$Wrong.play()
	
func destroy():
	$Destroy.play()
