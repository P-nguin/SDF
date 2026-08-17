# Game Design Document: 3D Spaceship Combat

## 1. Basic Premise
- **Core Gameplay:** 3D spaceship game where the player controls a spaceship to battle other spaceships.

## 2. Features

### 2.1. Complex Crew Mechanics
- **Equipment Operation:** Crewmates are required to actively operate ship equipment.
- **Traversal:** Crewmates must physically travel through the ship's corridors and layout to reach different rooms and stations.

### 2.2. Complex Camera System
To render a detailed view of the ship and its internals, a smart camera with multiple rendering modes is required.
- **Picture-in-Picture (PIP):** Located in the bottom right, this provides a basic overview for players to quickly assess the ship's status.
  - *External Mode:* Renders the exterior shell of the ship.
  - *Internal Mode:* Renders a skeletal view of the ship's internals.
- **Holographic Overlays:** Overlays a wireframe representation of the ship's internal structures.
- **Dynamic Cutaways:** An alternative visual option (to be explored as needed).

### 2.3. Advanced Combat & Damage Model
- **Voxel-Based Ships:** Ships are modeled using voxels to support a highly detailed, dynamic damage model.
- **Smooth Rendering:** Smooth voxel algorithms will be used to ensure the ship's exterior appears polished rather than blocky.
- **Weapon Specializations:** Different weapons will serve unique tactical roles (e.g., armor penetrating, equipment disabling, anti-crew, incendiary).
- **Multi-layered Fog of War:**
  - *Space Level:* The vastness of space conceals enemies. Players must use directional radar to scan ahead, which identifies ship types but doesn't fully reveal them visually.
  - *Ship Level:* Internal layouts of enemy ships are hidden. Players can use specialized radar to scan and reveal ship outlines, allowing for targeted subsystem attacks (e.g., "painting" a target on the external shell rather than simple click-to-target).
  - *Countermeasures:* Radar jammers can be deployed to disrupt incoming signals and prevent scanning.

### 2.4. Dynamic NavMesh Recalculation
- **Real-time Pathfinding Updates:** Because advanced combat can physically destroy parts of the ship (disrupting existing navigation meshes), the NavMesh must be recalculated dynamically.
- **SDF Integration:** Signed Distance Fields (SDF) will handle voxel data and mesh generation/updates.

### 2.5. Resource Management
- **Logistics:** Rooms and equipment require a steady flow of supplies and power.
- **Routing:** Players are tasked with managing and routing these resources efficiently across the ship during encounters.

### 2.6. Command System
- **Crew-Level Commands (Micro):** High-level directives delegated to crew members (e.g., "Control a weapon," "Fix the hull," "Put out a fire"). The system handles the low-level delegation to available crew.
- **Ship-Level Commands (Macro):** 
  - Standard RTS-style controls: Move, Hold Position, Hold Fire, Attack Move.
  - *Advanced Movement:* Click on the X/Z plane, then specify the Y plane (or use the scroll wheel). Orientation is set by dragging in the desired facing direction.
  - *Travel Orientation:* Depending on ship class, players can dictate specific orientations to maintain while traveling (e.g., keeping broadside weapons facing the enemy).

### 2.7. Voxel / Ship Editor
- **Customization:** A dedicated editor allowing players to design and modify both the exterior shell and the interior layout of their ships.

---

## 3. Entity & Component Models

### 3.1. Ship (Root Entity)
- **Transform:** Handles absolute position and rotation in the world space.
- **RigidBody:** Physics calculations and collision responses.
- **Metadata/Stats:** Core ship statistics (health, mass, speed, etc.).

### 3.2. VoxelChunk(s)
*Attached to the Ship entity.*
- **Transform:** Handles local offset relative to the ship's root transform.
- **SDFChunk:** Manages the actual distance values used to generate the 3D mesh.
- **LogicGrid:** Manages the simulation state of each cell (separated from SDFChunk to optimize cache performance).
- **Handle<Mesh>:** The rendered visual mesh.

### 3.3. Crewmate
*Entities navigating within the Ship.*
- **Transform:** Local position within the ship.
- **NavAgent:** Handles pathfinding and movement along the dynamic NavMesh.
- **TaskQueue:** Manages the list of assigned commands and actions.

### 3.4. ShipModule
*Interactive equipment and rooms within the Ship.*
- **Transform:** Local position.
- **Stats/Metadata:** Operational status, power draw, health, etc.
- **InteractionNode:** Designated physical points where crewmates must stand in order to operate the module.
