import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

// Auditor view: full trail with evidence links (issue #5 checklist).
Item {
    id: root
    property var bridge
    property var trail: []
    property string message: ""

    function refresh() {
        if (!bridge) return
        try {
            trail = JSON.parse(bridge.call("getAuditTrail", [])) || []
        } catch (e) { message = "sacco_core not reachable: " + e }
    }
    Component.onCompleted: refresh()

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 12

        Label { text: "Audit trail"; font.bold: true; font.pixelSize: 18 }
        Label { text: root.message; color: "firebrick"; visible: root.message !== "" }
        Button { text: "Refresh"; onClicked: root.refresh() }

        ListView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            model: root.trail
            delegate: Frame {
                width: ListView.view.width
                ColumnLayout {
                    width: parent.width
                    Label { text: modelData.kind + " — " + modelData.summary; font.bold: true }
                    Label { text: modelData.detail; visible: modelData.detail !== "" ; wrapMode: Text.Wrap; Layout.fillWidth: true }
                    Label {
                        text: "evidence: " + modelData.evidence_cid
                        visible: modelData.evidence_cid !== ""
                        color: "#2f6fdb"
                    }
                    Label { text: "recorded by " + modelData.recorded_by; font.italic: true }
                }
            }
        }
    }
}
