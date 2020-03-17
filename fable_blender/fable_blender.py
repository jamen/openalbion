#
# Addon info
#

bl_info = {
    'name': 'Blender for Fable',
	'description': 'Tool for supporting Fable assets in Blender.',
	'author': 'Jamen Marz <me@jamen.dev>',
	'license': 'ISC',
	'version': (0, 1, 0),
	'blender': (2, 81, 0),
	'location': 'View3D > Tools > Fable',
	'warning': '',
	'wiki_url': 'https://defable.netlify.com/fable_blender/index.html',
	'tracker_url': 'https://github.com/jamen/defable/issues',
	'link': '',
	'support': 'COMMUNITY',
	'category': '3D View'
}

import bpy

#
# Addon classes
#

class Fable_Workspace(bpy.types.Workspace):
	object_mode = 'EDIT'

class Fable_PT_Preferences(bpy.types.Panel):
	bl_idname = "Fable_PT_Preferences"
	bl_label = "Preferences"
	bl_space_type = "VIEW_3D"
	bl_region_type = "UI"
	bl_category = "Fable"

	bpy.types.Scene.fable_directory = bpy.props.StringProperty(
		name = "Fable Directory",
		description = "The file path to the Fable/Data/ directory.",
		maxlen = 512,
		subtype = 'DIR_PATH',
	)

	def draw(self, context):
		scene = context.scene
		layout = self.layout
		row = layout.row()
		column = row.column()
		column.prop(scene, 'fable_directory')
		column.operator("operator.select_fable_directory", icon='FILE_FOLDER')
		# layout.use_property_split = tracker_url
		# scene = context.scene
		# view = scene.view_settings
		# layout.prop(view, "view_transform")
		# layout.prop(view, "look")

#
# Register addon
#

classes = (
	Fable_Workspace
	Fable_PT_Preferences,
)

def register():
    for cls in classes:
        bpy.utils.register_class(cls)

def unregister():
    for cls in reversed(classes):
        bpy.utils.unregister_class(cls)