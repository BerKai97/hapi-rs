use std::borrow::Cow;

pub use crate::attribute::*;
use crate::errors::Result;
pub use crate::ffi::{
    raw::{AttributeOwner, CurveOrders, CurveType, GroupType, PartType},
    AttributeInfo, CurveInfo, GeoInfo, PartInfo,
};
use crate::node::HoudiniNode;
use crate::stringhandle::StringsArray;
use std::ffi::CString;

#[derive(Debug)]
pub struct Geometry<'session> {
    pub node: Cow<'session, HoudiniNode>,
    // TODO: Maybe revisit. GeoInfo may change and should be a get method
    pub info: GeoInfo<'session>,
}

impl<'session> Geometry<'session> {
    pub fn part_info(&'session self, id: i32) -> Result<PartInfo> {
        crate::ffi::get_part_info(&self.node, id).map(|inner| PartInfo { inner })
    }

    pub fn set_part_info(&self, info: &PartInfo) -> Result<()> {
        // TODO: Should part_id be provided by user or by PartInfo?
        crate::ffi::set_part_info(&self.node, info)
    }

    pub fn set_curve_info(&self, info: &CurveInfo, part_id: i32) -> Result<()> {
        crate::ffi::set_curve_info(&self.node, info, part_id)
    }

    pub fn set_curve_counts(&self, part_id: i32, count: &[i32]) -> Result<()> {
        crate::ffi::set_curve_counts(&self.node, part_id, count)
    }

    pub fn set_curve_knots(&self, part_id: i32, knots: &[f32]) -> Result<()> {
        crate::ffi::set_curve_knots(&self.node, part_id, knots)
    }

    pub fn set_vertex_list(&self, part_id: i32, list: impl AsRef<[i32]>) -> Result<()> {
        crate::ffi::set_geo_vertex_list(&self.node, part_id, list.as_ref())
    }

    pub fn set_face_counts(&self, part_id: i32, list: impl AsRef<[i32]>) -> Result<()> {
        crate::ffi::set_geo_face_counts(&self.node, part_id, list.as_ref())
    }

    pub fn geo_info(&'session self) -> Result<GeoInfo<'session>> {
        crate::ffi::get_geo_info(&self.node).map(|inner| GeoInfo {
            inner,
            session: &self.node.session,
        })
    }
    pub fn curve_info(&self, part_id: i32) -> Result<CurveInfo> {
        crate::ffi::get_curve_info(&self.node, part_id).map(|inner| CurveInfo { inner })
    }

    /// Retrieve the number of vertices for each curve in the part.
    pub fn curve_counts(&self, part_id: i32, start: i32, length: i32) -> Result<Vec<i32>> {
        crate::ffi::get_curve_counts(&self.node, part_id, start, length)
    }

    /// Retrieve the orders for each curve in the part if the curve has varying order.
    pub fn curve_orders(&self, part_id: i32, start: i32, length: i32) -> Result<Vec<i32>> {
        crate::ffi::get_curve_orders(&self.node, part_id, start, length)
    }

    /// Retrieve the knots of the curves in this part.
    pub fn curve_knots(&self, part_id: i32, start: i32, length: i32) -> Result<Vec<f32>> {
        crate::ffi::get_curve_knots(&self.node, part_id, start, length)
    }

    pub fn partitions(&self) -> Result<Vec<PartInfo>> {
        (0..self.info.part_count() + 1)
            .map(|i| self.part_info(i))
            .collect()
    }

    pub fn get_face_counts(&self, _info: &PartInfo) -> Result<Vec<i32>> {
        todo!()
        // crate::ffi::get_face_counts(&self.node, info.part_id(), info.face_count())
    }

    pub fn get_group_names(&self, group_type: GroupType) -> Result<StringsArray> {
        let count = match group_type {
            GroupType::Point => self.info.point_group_count(),
            GroupType::Prim => self.info.primitive_group_count(),
            _ => unreachable!("Impossible GroupType value"),
        };
        crate::ffi::get_group_names(&self.node, group_type, count)
    }

    pub fn get_attribute_names(
        &self,
        owner: AttributeOwner,
        part: &PartInfo,
    ) -> Result<StringsArray> {
        let counts = part.attribute_counts();
        let count = match owner {
            AttributeOwner::Invalid => panic!("Invalid AttributeOwner"),
            AttributeOwner::Vertex => counts[0],
            AttributeOwner::Point => counts[1],
            AttributeOwner::Prim => counts[2],
            AttributeOwner::Detail => counts[3],
            AttributeOwner::Max => unreachable!(),
        };
        crate::ffi::get_attribute_names(&self.node, part.part_id(), count, owner)
    }

    pub fn get_attribute<T: AttribDataType>(
        &self,
        part_id: i32,
        owner: AttributeOwner,
        name: &str,
    ) -> Result<Option<Attribute<T>>> {
        let name = std::ffi::CString::new(name)?;
        let inner = crate::ffi::get_attribute_info(&self.node, part_id, owner, &name)?;
        if inner.exists < 1 {
            return Ok(None);
        }
        let attrib = Attribute::new(name, AttributeInfo { inner }, &self.node);
        Ok(Some(attrib))
    }

    pub fn add_attribute<T: AttribDataType>(
        &self,
        name: &str,
        part_id: i32,
        info: &AttributeInfo,
    ) -> Result<Attribute<T>> {
        let name = CString::new(name)?;
        crate::ffi::add_attribute(&self.node, part_id, &name, &info.inner)?;
        Ok(Attribute::new(
            name,
            AttributeInfo { inner: info.inner },
            &self.node,
        ))
    }

    pub fn add_group(&self, part_id: i32, group_name: &str, group_type: GroupType) -> Result<()> {
        let group_name = CString::new(group_name)?;
        crate::ffi::add_group(
            &self.node.session,
            self.node.handle,
            part_id,
            group_type,
            &group_name,
        )
    }

    pub fn set_group_membership(
        &self,
        part_id: i32,
        group_type: GroupType,
        group_name: &str,
        array: &[i32],
    ) -> Result<()> {
        let group_name = CString::new(group_name)?;
        crate::ffi::set_group_membership(
            &self.node.session,
            self.node.handle,
            part_id,
            group_type,
            &group_name,
            array,
        )
    }

    pub fn get_group_membership(
        &self,
        part_info: Option<&PartInfo>,
        group_type: GroupType,
        group_name: &str,
    ) -> Result<Vec<i32>> {
        let group_name = CString::new(group_name)?;
        let tmp;
        let part = match part_info {
            None => {
                tmp = self.part_info(0)?;
                &tmp
            }
            Some(part) => part,
        };
        crate::ffi::get_group_membership(
            &self.node.session,
            self.node.handle,
            part.part_id(),
            group_type,
            &group_name,
            part.element_count_by_group(group_type),
        )
    }

    pub fn group_count_by_type(&self, group_type: GroupType) -> i32 {
        crate::ffi::get_group_count_by_type(&self.info, group_type)
    }

    pub fn save_to_file(&self, filepath: &str) -> Result<()> {
        let path = CString::new(filepath)?;
        crate::ffi::save_geo_to_file(&self.node, &path)
    }

    pub fn commit(&self) -> Result<()> {
        crate::ffi::commit_geo(&self.node)
    }
}

impl PartInfo {
    pub fn element_count_by_group(&self, group_type: GroupType) -> i32 {
        crate::ffi::get_element_count_by_group(self, group_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::tests::with_session;

    #[test]
    fn geometry_triangle() {
        with_session(|session| {
            let node = session.create_input_node("test").expect("input node");
            let geo = node.geometry().expect("geometry").unwrap();

            let part = PartInfo::default()
                .with_part_type(PartType::Mesh)
                .with_face_count(1)
                .with_point_count(3)
                .with_vertex_count(3);
            geo.set_part_info(&part).expect("part_info");
            let info = AttributeInfo::default()
                .with_count(part.point_count())
                .with_tuple_size(3)
                .with_owner(AttributeOwner::Point)
                .with_storage(StorageType::Float);
            let attr_p = geo
                .add_attribute::<f32>("P", part.part_id(), &info)
                .unwrap();
            attr_p
                .set(
                    part.part_id(),
                    &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0],
                )
                .unwrap();
            geo.set_vertex_list(0, [0, 1, 2]).unwrap();
            geo.set_face_counts(0, [3]).unwrap();
            geo.commit().expect("commit");

            node.cook_blocking(None).expect("cook");

            let val: Vec<_> = attr_p.read(part.part_id()).expect("read_attribute");
            assert_eq!(val.len(), 9);
        });
    }
}
