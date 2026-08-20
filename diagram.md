```mermaid

flowchart TD
    %% ==========================================
    %% PLAYER INPUTS
    %% ==========================================
    subgraph Inputs [Player Inputs]
        LeftClick(["Left Click"])
        V_Key(["Press 'V'"])
        F_Key(["Press 'F'"])
    end

    %% ==========================================
    %% CAMERA & TARGETING
    %% ==========================================
    subgraph Camera [Camera & Targeting System]
        CamEntity(["Camera Entity"])
        PanState["PanOrbitState<br/>(Tracks Focus, Pitch, Yaw)"]
        Raycast{{"SpatialQuery.cast_ray()"}}

        CamEntity -.->|Has| PanState
        LeftClick -->|Triggers| Raycast
        Raycast -.->|Casts from| CamEntity
    end

    %% ==========================================
    %% ROOT SHIP ENTITY
    %% ==========================================
    subgraph RootShip [Root Ship Entity]
        ShipE(["Ship Entity"])
        RB["RigidBody::Dynamic<br/>GravityScale(0.0)"]
        SDF["SDFChunks<br/>(Exterior & Interior Distances)"]
        Remesh["NeedsRemesh Flag"]
        Render["ShipRender<br/>(Exterior/Interior/Both)"]
        SelectedTag["Selected Component"]

        ShipE -.->|Has| RB
        ShipE -.->|Has| SDF
        ShipE -.->|Has| Remesh
        ShipE -.->|Has| Render
    end

    %% ==========================================
    %% MESH GENERATION PIPELINE
    %% ==========================================
    subgraph Generation [Procedural Generation Pipeline]
        GenSystem[["generate_chunk_mesh_system"]]
        QEF{{"QEFData Solver"}}
        MeshBuilder(("Mesh Builder<br/>(Vertices, Normals, Indices)"))

        Remesh -->|Triggers| GenSystem
        SDF -->|Provides Distances| GenSystem
        GenSystem <-->|Solves Optimal Vertices| QEF
        GenSystem -->|Generates Data| MeshBuilder
    end

    %% ==========================================
    %% CHILD MESH ENTITIES
    %% ==========================================
    subgraph ChildMeshes [Child Mesh Entities]
        ExtEntity(["Exterior Entity"])
        IntEntity(["Interior Entity"])

        ExtMeta["GeneratedShipMesh<br/>(Kind: Exterior)"]
        IntMeta["GeneratedShipMesh<br/>(Kind: Interior)"]

        ExtCol["Collider::try_trimesh"]

        Vis["Visibility & Material Alpha"]

        ExtEntity -.->|Has| ExtMeta
        ExtEntity -.->|Has| ExtCol
        ExtEntity -.->|Has| Vis

        IntEntity -.->|Has| IntMeta
        IntEntity -.->|Has| Vis
    end

    %% ==========================================
    %% RELATIONSHIPS & FLOWS
    %% ==========================================

    %% Spawning Flow
    MeshBuilder -->|Spawns| ExtEntity & IntEntity
    ShipE -->|Parent Of| ExtEntity & IntEntity
    GenSystem -->|Removes| Remesh

    %% Selection Flow
    Raycast -->|Hits| ExtCol
    Raycast -->|Reads| ExtMeta
    ExtMeta -.->|ship: Entity points to| ShipE
    LeftClick -->|Attaches on hit| SelectedTag
    SelectedTag -.->|Attached to| ShipE

    %% X-Ray / Render Toggle Flow
    V_Key -->|Requires| SelectedTag
    V_Key -->|Cycles| Render
    Render -->|Updates| Vis

    %% Camera Focus Flow
    F_Key -->|Requires| SelectedTag
    F_Key -->|Sets focus: Entity| PanState
    PanState -.->|Tracks| ShipE

    %% Styling
    classDef entity fill:#2a4d69,stroke:#4b86b4,stroke-width:2px,color:#fff
    classDef component fill:#4b86b4,stroke:#adcbe3,stroke-width:1px,color:#fff
    classDef system fill:#63ace5,stroke:#e7eff6,stroke-width:2px,color:#000
    classDef input fill:#e7eff6,stroke:#63ace5,stroke-width:2px,color:#000,shape:stadium
    classDef tag fill:#f7c06d,stroke:#f18904,stroke-width:2px,color:#000

    class ShipE,ExtEntity,IntEntity,CamEntity entity
    class RB,SDF,Remesh,Render,ExtMeta,IntMeta,ExtCol,Vis,PanState component
    class GenSystem,QEF,MeshBuilder,Raycast system
    class LeftClick,V_Key,F_Key input
    class SelectedTag tag

```

````mermaid
flowchart TD
    %% Styling definitions
    classDef plugin fill:#1b2a49,stroke:#465c8b,stroke-width:2px,color:#fff
    classDef entity fill:#2a4d69,stroke:#4b86b4,stroke-width:2px,color:#fff
    classDef component fill:#4b86b4,stroke:#adcbe3,stroke-width:1px,color:#fff
    classDef system fill:#63ace5,stroke:#e7eff6,stroke-width:2px,color:#000
    classDef ui fill:#465c8b,stroke:#adcbe3,stroke-width:2px,color:#fff

    %% ==========================================
    %% CAMERA MODULE
    %% ==========================================
    subgraph CameraModule ["📂 camera/mod.rs (Camera Plugin)"]
        direction TB
        CamEntity(["Commander Camera Entity"])
        CamComp["CommanderCamera<br/>(RenderState, InputMode)"]
        PanComp["PanOrbitState<br/>(Radius, Focus)"]

        SysCalcZoom[["System: decide_render_mode"]]
        SysInput[["System: handle_camera_input"]]

        CamEntity -.->|Owns| CamComp & PanComp
        SysCalcZoom -->|Reads| PanComp
        SysCalcZoom -->|Updates| CamComp
    end

    %% ==========================================
    %% SHIP MODULE
    %% ==========================================
    subgraph ShipModule ["📂 ship/mod.rs (Ship Plugin)"]
        direction TB
        ShipRoot(["Ship Root Entity"])
        SDFComp["SDFChunks (Voxels)"]
        AsciiComp["AsciiRepresentation"]

        ChildMesh(["Child: 3D Mesh Entity"])
        ChildText(["Child: Text2d Entity"])
        VisMesh["Visibility (Mesh)"]
        VisText["Visibility (Text)"]

        SysSpawn[["System: spawn_ship"]]

        ShipRoot -.->|Owns| SDFComp & AsciiComp
        SysSpawn -->|Constructs| ShipRoot
        SysSpawn -->|Spawns| ChildMesh & ChildText
        ChildMesh -.->|Owns| VisMesh
        ChildText -.->|Owns| VisText
    end

    %% ==========================================
    %% LOD / RENDER RULES MODULE (The Glue)
    %% ==========================================
    subgraph RenderModule ["📂 tactical_render/mod.rs (Tactical Plugin)"]
        SysLOD[["System: apply_tactical_lod"]]
    end

    %% ==========================================
    %% TERMINAL MODULE
    %% ==========================================
    subgraph TerminalModule ["📂 terminal/mod.rs (UI Plugin)"]
        UIEgui[/"Egui Terminal Window"/]
        SysRenderUI[["System: draw_terminal_ui"]]
        SysParse[["System: parse_commands"]]
    end

    %% ==========================================
    %% RELATIONSHIPS (The "Everything Else")
    %% ==========================================

    %% LOD System connects Camera Data to Ship Data
    CamComp -.->|Read State by| SysLOD
    SysLOD -->|Toggles| VisMesh
    SysLOD -->|Toggles| VisText

    %% Terminal reacts to Camera Input Mode
    CamComp -.->|Read Mode by| SysRenderUI
    SysRenderUI -->|Draws| UIEgui
    UIEgui -->|Submits string to| SysParse
    SysParse -->|Pushes to ShipCommandQueue| ShipRoot

    class CameraModule,ShipModule,RenderModule,TerminalModule plugin
    class CamEntity,ShipRoot,ChildMesh,ChildText entity
    class CamComp,PanComp,SDFComp,AsciiComp,VisMesh,VisText component
    class SysCalcZoom,SysInput,SysSpawn,SysLOD,SysRenderUI,SysParse system
    class UIEgui ui
    ```
````
