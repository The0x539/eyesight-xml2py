import bpy
import os.path
from .node_dsl import NodeGraph
from .custom_nodes import node_group_uv_degradation, node_group_project_to_axis_plane

def load_image(filename: str | None) -> bpy.types.Image:
    assert filename != ""

    if img := bpy.data.images.get(filename or "blank"):
        return img

    if filename is None:
        img = bpy.data.images.new("blank", 1, 1, alpha=True)
        img.pixels = (0.0, 0.0, 0.0, 0.0)
        img.update()
    else:
        # TODO
        eyesight_path = "C:/Program Files/Studio 2.0/PhotoRealisticRenderer/win/64"
        img_path = os.path.join(eyesight_path, filename)
        # TODO: does check_existing reuse image objects (desired)
        # or does it only reuse the underlying buffer, creating a new Image no matter what?
        img = bpy.data.images.load(img_path, check_existing=True)

    return img

